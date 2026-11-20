// Optional "Login with lostark.bible" (OAuth public client + PKCE); see AI notes "lostark.bible OAuth API".
import * as oauth from "oauth4webapi";
import { computed, shallowRef } from "vue";
import { StorageSerializers, useStorage } from "@vueuse/core";
import transliterate from "@sindresorhus/transliterate";

export const BIBLE = "https://lostark.bible";
const AS: oauth.AuthorizationServer = {
  issuer: BIBLE,
  authorization_endpoint: `${BIBLE}/oauth/authorize`,
  token_endpoint: `${BIBLE}/oauth/token`,
  revocation_endpoint: `${BIBLE}/oauth/revoke`,
};
const client = (): oauth.Client => ({
  client_id: import.meta.env.VITE_BIBLE_CLIENT_ID,
});
const SCOPES = "identify rosters loadouts";
const PENDING_KEY = "bible_auth_pending";
const redirect_uri = () => `${location.origin}/auth/callback`;

export interface BibleAuth {
  token: string;
  expires_at: number;
  name: string;
  avatar?: string;
}
export interface BibleCharacter {
  name: string;
  class: string;
  ilvl: number;
}
export interface BibleRoster {
  region: "NA" | "CE";
  world: string;
  characters: BibleCharacter[];
}

// kept apart from saved configs so it never ends up in exports or uploads
export const bible_auth = useStorage<BibleAuth | null>(
  "bible_auth",
  null,
  localStorage,
  {
    serializer: StorageSerializers.object,
  },
);
export const signed_in = computed(
  () => bible_auth.value !== null && bible_auth.value.expires_at > Date.now(),
);

export async function sign_in() {
  const verifier = oauth.generateRandomCodeVerifier();
  const state = oauth.generateRandomState();
  sessionStorage.setItem(
    PENDING_KEY,
    JSON.stringify({
      verifier,
      state,
      back: location.pathname + location.search,
    }),
  );
  const url = new URL(AS.authorization_endpoint);
  url.search = new URLSearchParams({
    client_id: client().client_id,
    redirect_uri: redirect_uri(),
    response_type: "code",
    scope: SCOPES,
    state,
    code_challenge: await oauth.calculatePKCECodeChallenge(verifier),
    code_challenge_method: "S256",
  }).toString();
  location.assign(url);
}

// finishes the sign-in on /auth/callback; returns the path to go back to
export async function handle_callback(): Promise<string> {
  const pending = JSON.parse(sessionStorage.getItem(PENDING_KEY));
  sessionStorage.removeItem(PENDING_KEY);
  const params = oauth.validateAuthResponse(
    AS,
    client(),
    new URL(location.href),
    pending.state,
  );
  const response = await oauth.authorizationCodeGrantRequest(
    AS,
    client(),
    oauth.None(),
    params,
    redirect_uri(),
    pending.verifier,
  );
  const tokens = await oauth.processAuthorizationCodeResponse(
    AS,
    client(),
    response,
  );
  const user = await bible_get("/user", tokens.access_token);
  bible_auth.value = {
    token: tokens.access_token,
    expires_at: Date.now() + tokens.expires_in * 1000,
    name: user.discordGlobalName ?? user.discordUsername,
    // Discord's default avatar when none is set
    avatar: user.discordAvatar
      ? `https://cdn.discordapp.com/avatars/${user.discordId}/${user.discordAvatar}.png?size=64`
      : `https://cdn.discordapp.com/embed/avatars/${(BigInt(user.discordId) >> 22n) % 6n}.png`,
  };
  clear_rosters();
  return pending.back;
}

export function sign_out() {
  const token = bible_auth.value?.token;
  bible_auth.value = null;
  clear_rosters();
  if (token)
    oauth.revocationRequest(AS, client(), oauth.None(), token).catch(() => {});
}

async function bible_get(
  path: string,
  token = bible_auth.value?.token,
): Promise<any> {
  const response = await fetch(`${BIBLE}/api/oauth${path}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (response.status === 401) {
    bible_auth.value = null;
    clear_rosters();
  }
  if (!response.ok) throw new Error(`lostark.bible ${response.status}`);
  return response.json();
}

// names typed by hand may lack the in-game accents (Elsa for Ëlsa)
export const norm_name = (name: string) => transliterate(name).toLowerCase();

// exact name first: a roster may hold both an accented name and its plain form
export function match_name<T>(
  items: T[],
  name: string,
  key: (x: T) => string,
) {
  return (
    items.find((x) => key(x) === name) ??
    items.find((x) => norm_name(key(x)) === norm_name(name))
  );
}

// linked rosters change on lostark.bible: refetched on sync and after a failed API fetch
export const bible_rosters = shallowRef<BibleRoster[] | null>(null);
let rosters_promise: Promise<BibleRoster[]> | null = null;
function clear_rosters() {
  rosters_promise = null;
  bible_rosters.value = null;
}
export function linked_rosters(reload = false): Promise<BibleRoster[]> {
  if (reload) rosters_promise = null;
  if (rosters_promise === null) {
    const p: Promise<BibleRoster[]> = bible_get("/rosters").then((x) => {
      if (rosters_promise === p) bible_rosters.value = x.rosters;
      return x.rosters;
    });
    p.catch(() => {
      if (rosters_promise === p) rosters_promise = null;
    });
    rosters_promise = p;
  }
  return rosters_promise;
}

// reactive lookup in the loaded list (starts the load; null until it lands)
export function linked_now(region: string, char_name: string) {
  if (!signed_in.value) return null;
  if (bible_rosters.value === null) {
    linked_rosters().catch(() => {});
    return null;
  }
  const pairs = bible_rosters.value
    .filter((roster) => roster.region === region)
    .flatMap((roster) =>
      roster.characters.map((character) => ({ roster, character })),
    );
  return match_name(pairs, char_name, (x) => x.character.name) ?? null;
}

// the signed-in user's roster holding this character, or null (signed out, not linked, or an error)
export async function find_linked(region: string, char_name: string) {
  if (!signed_in.value) return null;
  try {
    await linked_rosters();
  } catch {
    return null; // falls back to public scraping
  }
  return linked_now(region, char_name);
}

export const get_loadouts = (region: string, char_name: string) =>
  bible_get(`/loadouts/${encodeURIComponent(char_name)}?region=${region}`);
