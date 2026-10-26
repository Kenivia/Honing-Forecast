# Community

What the community has on income per piece of content (Chaos Rift, Kurzan Front, Guardian Raid, cube, hourglass, hell, raid bonus loot): every site looked at, what it had or lacked, and its tables as read, so they can be cross-checked. Gathered 2026-10-25 and 10-26. Nothing here is used by the site yet.

## Sites

| Site | Region | Has | Lacks / notes |
| --- | --- | --- | --- |
| Tylobic, "Raids, Dungeons & Guardians Rewards" (Google Sheet `1YQpWt8iOK6yO5_7r3rvZZKkoRy8Z0aEAPHy11gYZZQ8`; `/export?format=xlsx` works without login) | Global, EUC prices | Per-run averages for Chaos Rift 1730–1770, Kurzan Front 1640–1720, Guardian Raids, T2/T3 chaos silver; raid gold and bonus loot; market prices (EUC, 2026-10-22) | No sample sizes ("averages, there can be a lot of variation"); corrections by Discord DM. The xlsx export turns ranges like `11-12` into dates. |
| lostmokokos.com (`/daily`, `/raids`, `/gold`) | Global | Renders Tylobic's sheet ("last updated September 2026") with its own prices | Nothing beyond the sheet; price source not named |
| LoaGuard, loaguard.com/content-rewards/ | KR | Rift, Kurzan Front, Guardian, Chaos Gate, field boss, Haal's Hourglass (by sand count), Ebony Cube; 1700–1770. Notes the source of each row (실측, official gem odds, ratio from the tier below) | Values shown rounded (1천G); the per-row "참조 가치표" (silver) is not explained; JS only |
| LoaGuard, loaguard.com/hell-rewards/ | KR | Hell / Abyss chest contents by ilvl and stage; per-character exclusions | No per-run totals; no drop rates |
| 껨산기, gcalc.kr/content-rewards | KR | Kurzan Front and Rift 1640–1750, Guardian 1640–1750, cube 1–4, hourglass 1–2; unit price per item; tradable vs total | No 1770; excludes accessories, relic books, 편린; JS only |
| LOAGAP, loagap.com/efficiency/chaos | KR | Kurzan Front 1640–1720 with contents | Looks stale (stones tradable, about half gcalc's quantities); no rift, guardian; rested assumed 2x |
| loagg.com (`/hell`, `/raid`, `/todo`) | KR | Hell bound gold per stage; raid gold and bonus-loot value (Open API prices) | Hell chest quantities did not load; no rift / guardian page; `/todo` has no quantities |
| xloa.gg | KR | Nothing readable | JS only; fetch returned the title only |
| loakraken (GitHub `loakraken/loakraken`, `loakraken-web`) | KR | Open source; `lib/tools/data/` holds `cube-rewards.json`, `raid-rewards.json`, `more-reward.json`; hell-reward verification JSON | No rift, Kurzan or guardian data |
| Official notices, lostark.game.onstove.com `/News/Notice/Views/13326` (2025-12-24), `13339` (2026-01-07) | KR | Rules: costs, limits, bound status | No quantities, no rested multiplier |
| Fandom wiki "Chaos_Rift"; Lost Ark on X | Global | Rules: 1730+, 100 Aura + 40 rest bonus, once a day per character | No quantities |
| Inven 4821/93735 (편린 drop statistics) | KR | 편린 roughly 8% a run, doubled when rested; outcome split guessed at 20% each | Old, small sample (10 characters, a month) |
| zockify.com, lostark-helper.com, maxroll, mobalytics, icy-veins, Steam threads | Global | Nothing current | zockify is 2022; lostark-helper reset the connection; Steam threads are anecdotes ("rested is not exact double") |
| namu.wiki | KR | Not read | 403 to automated fetches |
| Lost Ark Fandom wiki (`lostark.fandom.com`) | Global | Playtime Reward, Waymark Mission, Ebony Cube (tiers, guild ticket prices), Arktoberfest, patch pages with the 2026 gold cuts | Pages give 402 to fetches; `/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&format=json&titles=<Page>` returns the wikitext |
| Maxroll paradise guide, Inven Global 25706, vortexgaming.io 552449 | Global | Paradise rules (descents, altars, falling point); Vortex has floor-100 odds per key grade | No sampled floor distribution anywhere; Vortex's look computed |

The raw research of 2026-10-26 (the fresh Tylobic xlsx, the client queries, per-gate raid tables, the shop dump) is in `ignored/income_research_2026-10-26/`.

## Rules (official notices, wiki)

- Chaos Rift (혼돈의 균열, 심연의 역류 I/II/III at 1730/1750/1770), solo, timed. Shares 공명의 기운 (Aura of Resonance) and 휴식 보너스 (rest bonus) with Chaos Dungeon and Kurzan Front. One entry costs 100 Aura; holding 40+ rest bonus spends 40 and "increases rewards" (no multiplier given). Its T4-upper honing materials are character bound. 운명의 편린 drops at some chance and gives tradable T4-upper materials or other rewards.
- 시련의 모래 (Sand of Trial) drops from the rift at some chance, at most 5 held; clearing Haal's Hourglass (weekly, per character) spends all of it and enhances the rewards by the count.
- 가디언의 잔영 (Guardian Shade) 1730+: the same reward kinds as Guardian Raid, gems and 편린 at some chance.

## Chaos Rift, per run (not rested)

| ilvl | Source | Destruction crystal | Guardian crystal | Great leapstone | Shards | Silver | Hero gem | Rare gem | Common gem | Sand |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1730 | LoaGuard = gcalc | 313 | 1,116 | 14.9 | 38,078 | 205,750 | 0.125 | 0.375 | 2 | 0.2 (stage 1) |
| 1730 | Tylobic | 313 | 1,035 | 16 | 42,447 | 211,350 | | | | |
| 1750 | LoaGuard = gcalc | 369 | 1,229 | 20 | 52,600 | 230,926 | 0.2 | 0.3 | 1.2 | 0.2 (stage 2) |
| 1750 | Tylobic | 377 | 1,160 | 21 | 52,297 | 232,565 | | | | |
| 1770 | LoaGuard | 420 | 1,200 | 24 | 54,000 | 250,000 | 0.2 | 0.3 | 1.2 | 0.2 |
| 1770 | Tylobic | 469 | 1,422 | 27 | 63,747 | 232,871 | | | | |

Tylobic's gold value (sheet, EUC 2026-10-22, normal / rested): 1730 14,957 / 29,914; 1750 17,702 / 35,403; 1770 21,835 / 43,669. Lost Mokokos showed 15,927 / 31,853 (1730) and 19,086 / 38,172 (1750) at its own prices.

gcalc totals (2026-10-25): 1730 tradable 1,294, total 20,543; 1750 tradable 1,294, total 24,918. The tradable part is the sand's gems, not the run's own drops.

## Kurzan Front, per run (not rested)

| ilvl | Source | Destruction | Guardian | Leapstone | Shards | Silver | Cube ticket | Hero / rare / common gem |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1640 | gcalc | 332 | 899 | 19.0 | 21,989 | 180,002 | 0.2 (unlock 1) | |
| 1640 | Tylobic | 366 | 934 | 21 | 17,767 | 175,468 | | |
| 1640 | LOAGAP | 150.6 | 458 | 11 | 21,000 | | | lvl-1 gem 3 |
| 1660 | gcalc | 362 | 1,120 | 28.7 | 28,980 | 183,110 | 0.2 (1) | |
| 1660 | Tylobic | 345 | 1,170 | 27 | 23,100 | 180,143 | | |
| 1660 | LOAGAP | 185 | 522 | 14 | 28,505 | | | lvl-1 gem 4 |
| 1680 | gcalc | 454 | 1,318 | 36.4 | 31,290 | 194,203 | 0.2 (2) | |
| 1680 | Tylobic | 475 | 1,274 | 35 | 33,684 | 197,508 | | |
| 1680 | LOAGAP | 222 | 640 | 18.2 | 31,210 | | | lvl-1 gem 5 |
| 1700 | gcalc = LoaGuard | 512 | 1,601 | 42.4 | 34,442 | 198,990 | 0.2 (3) | 0.075 / 0.225 (gcalc 0.23) / 1.2 |
| 1700 | Tylobic | 630 | 1,671 | 41 | 33,510 | 192,386 | | |
| 1700 | LOAGAP | 283 | 810 | 21.8 | 33,664 | | | lvl-1 gem 5.8 |
| 1720 | gcalc = LoaGuard | 757 | 2,137 | 46 | 39,012 | 202,260 | 0.2 (4) | 0.1 / 0.3 / 1.6 |
| 1720 | Tylobic | 674 | 2,181 | 48 | 42,654 | 200,158 | | |
| 1720 | LOAGAP | 343 | 1,021 | 27.5 | 38,922 | | | lvl-1 gem 6.9 |

Totals:
- Tylobic (EUC 10-22, normal / rested): 1640 3,695 / 7,390; 1660 4,245 / 8,490; 1680 5,706 / 11,412; 1700 6,673 / 13,347; 1720 7,983 / 15,966. Lost Mokokos: 4,520, 5,206, 7,171, 8,197, 9,783.
- gcalc (10-25, tradable / total): 1640 259 / 8,682; 1660 259 / 9,532; 1680 518 / 10,636; 1700 690 / 13,692; 1720 949 / 16,162.
- LOAGAP (10-26, tradable / bound / total): 1640 836 / 1,769 / 2,605; 1660 1,063 / 2,398 / 3,461; 1680 1,310 / 2,635 / 3,945; 1700 1,580 / 2,848 / 4,428; 1720 1,907 / 3,300 / 5,207. An earlier fetch (10-25) read 864 / 1,818 / 2,682 at 1640 and so on, at that day's prices. LOAGAP lists the stones as tradable and shards as "파편 주머니(대)" counted in shards.

## Guardian Raid, per clear (not rested)

| ilvl | Guardian | Source | Silver | Lvl-1 gems | Hero / rare / common gem | Other |
| --- | --- | --- | --- | --- | --- | --- |
| 1640 | Argeos | gcalc | 33,707 | 2.8 | | |
| 1640 | | Tylobic | 34,976 | 2–3 | | |
| 1680 | Skolakia | gcalc | 39,920 | 4.1 | | |
| 1680 | | Tylobic | 40,687 | 4–5 | | |
| 1700 | Drextalas | gcalc = LoaGuard | 43,982 | 4.7 | 0.075 / 0.225 / 1.2 | |
| 1700 | | Tylobic | 46,295 | 2–6 | | |
| 1720 | Krathios | gcalc = LoaGuard | 49,890 | 6.3 | 0.175 / 0.525 / 2.8 | |
| 1720 | | Tylobic | 50,275 | 6–7 | | |
| 1730 | Shade I | gcalc = LoaGuard | 53,767 | 10.7 | 0.15 / 0.5 / 3 | |
| 1730 | | Tylobic | 55,000 | 10–11 | | |
| 1750 | Shade II | gcalc = LoaGuard | 59,950 | 12 | 0.2 / 0.6 / 3.2 | |
| 1750 | | Tylobic | 57,500 | 11–12 | | |
| 1770 | Shade III | LoaGuard | 60,000 | 5.5 lvl-2 gem boxes | 0.25 / 0.75 / 4 | |
| 1770 | | Tylobic | 61,500 | 15–18 | | |

- LoaGuard 1770 notes: gem boxes "실측 5~6개 · 휴식 시 같은 수량 추가(총 10~12개)" (measured 5–6, rested adds as many again); hero and rare gems "하위 구간 보상 비율로 추정" (estimated from the tier below); common gems "실측 평균" (measured); silver "휴식 기준 11~13만 실측 ÷ 2" (measured rested, halved).
- Tylobic's other guardian columns (1640–1770, read through Lost Mokokos's headers): ability stones 0–2, bracelets 0–2 (0–3 at 1770), fragments 5 / 10 / 15 / 20 / 35 / 40 / 60 (1640 → 1770), 10–15 accessories. Lost Mokokos's own page was read as showing ability stones 1–2.
- gcalc totals (tradable / total): 1640 403 / 638; 1680 590 / 868; 1700 676 / 3,019; 1720 906 / 6,005; 1730 1,539 / 5,993; 1750 1,726 / 7,574.

## Ebony Cube (per ticket) and Haal's Hourglass (per clear)

gcalc and LoaGuard have the same quantities.

| Content | Lvl-1 gems | Leapstone | Lava breath | Glacier breath | Card XP | Silver |
| --- | --- | --- | --- | --- | --- | --- |
| Cube unlock 1 | 9 | 14 (T4) | 4 | 4 | 14,000 | 140,000 |
| Cube unlock 2 | 18 | 25 | 5 | 5 | 14,500 | 150,000 |
| Cube unlock 3 | 24 | 32 | 6 | 6 | 15,000 | 160,000 |
| Cube unlock 4 | 33 | 41 | 8 | 8 | 15,500 | 180,000 |
| Hourglass 1 (1730) | 45 | 10 (great) | 10 | 10 | 16,000 | 200,000 |
| Hourglass 2 (1750) | 54 | 12 (great) | 12 | 12 | 16,500 | 211,689 |

- **Hourglass 3 (1770), LoaGuard, with s sands spent:** each item is its base times (1 + s), s from 0 to 5. The base is 7 lvl-3 gem boxes, 15 great leapstones, 6 태초의 조각, 1 stage-3 engraving pouch, and 14 of each breath.
- **Card XP dominates the totals:** gcalc values it at 1.2 g a point. Cube 1 totals 22,038 (tradable 1,310), and 16,262 of that is card XP. LoaGuard leaves card XP out by default (a toggle).
- **How a sand is valued:** gcalc's rift row values a sand at about one hourglass, 35,502 against hourglass 1's total of 35,360. That fits the (1 + s) scaling.

## Chaos Gate and field boss (LoaGuard)

- **Chaos Gate 1730:** 3,500 bound gold, 4 large shard pouches, 6 lvl-1 gems, 6 of each breath, 6,500 card XP.
- **Chaos Gate 1770:** 7 bound gold bars (1,000 each), 8 lvl-1 gems, 1 soul leaf, 8 of each breath, 1 rare card pack II (valued at 0 g, as are all card packs of hero grade and below), 5 large shard pouches, 2 태초의 조각.
- **Field boss 1730:** 21 lvl-1 gems, 470 destruction crystals, 1,583 guardian crystals, 39 great leapstones, 3 of each breath, 2,500 card XP.
- **Sevek Atun 1770:** 1,026 destruction crystals, 3,018 guardian crystals, 63 great leapstones, 3 lava and 4 glacier breath, 3 soul leaves, 1 태초의 조각, 1 lvl-3 gem.

## Hell / Abyss and raid bonus loot

LoaGuard hell, 1750, stage 0, per chest:
- **Base materials in every chest:** 5,200 shards, 26 destruction crystals, 170 guardian crystals, 4 great leapstones. Abundance (풍요, hell only) multiplies these by 10.
- **Each chest's own reward:**
  - 12 lava / 36 glacier breath
  - 28 transferred leapstones
  - 75 superior Abidos fusion
  - a choice of 600 destruction or 1,800 guardian crystals
  - 9 destiny stones / 7 chaos stones
  - 4 bracelets
  - 5,500 bound gold
  - 3 rare gem choices
  - 9 ability stone kits
  - 30 great leapstones

loagg hell bound gold per stage:

| ilvl | Stage 0 | Stage 5 | Stage 10 |
| --- | --- | --- | --- |
| 1640 | 3,200 | 16,000 | 96,000 |
| 1700 | 3,500 | 18,000 | 108,000 |
| 1730 | 4,500 | 22,000 | 130,000 |
| 1750 | 5,500 | 26,400 | 156,000 |

loagg raid bonus loot (clear gold / bonus cost / material value / net):
- **1780:** 벨가르딘 75,000 / 24,000 / 48,346 / +24,346
- **1770:** 벨가르딘 hard 62,000 / 19,840 / 41,995 / +22,155
- **1750:** 지평 3 and 벨가르딘 normal 50,000 / 16,000 / 33,878 / +17,878
- **1740:** 세르카 nightmare 54,000 / 17,280 / 33,878 / +16,598
- **1730:**
  - 종막 hard 48,000 / 15,360 / 28,219 / +12,859
  - 세르카 hard 44,000 / 14,080 / 27,691 / +13,611
- **1720:** 지평 2 40,000 / 12,800 / 12,979 / +179
- **1710:**
  - 종막 normal 32,000 / 10,240 / 10,165 / −75
  - 세르카 normal 32,000 / 10,240 / 10,810 / +570
- **1700:**
  - 지평 1 30,000 / 9,600 / 10,730 / +1,130
  - 모르둠 hard 27,000 / 8,350 / 18,604 / +10,254
- **1680:** 모르둠 normal 21,000 / 7,010 / 5,515 / −1,495

## Raids (Global)

- **The client has every gate** (`Screen scanner/Game files.md`, "Income in the client"); Tylobic's sheet (October 2026, Belgardin included) agrees with it on every gate but three: Serca Normal's bonus chest price (sheet 4,480 / 6,720, client 4,160 / 6,080), Kazeros Normal's gold per gate (sheet has Serca's 6,500 / 9,500, client 5,500 / 10,500; same totals) and Behemoth's bonus chest price (sheet 1,250 / 2,000, client 720 / 1,630). Take the client.
- **Global totals equal KR's** (loagg) for every raid compared, after Global's cuts of 2026-06-10 and 09-16 (wiki patch pages). Normal, Solo and Matching pay half roster bound; Hard and up pay tradable; Horizon Cathedral character bound; Thaemine and Echidna Normal roster bound.

## Other weekly and daily sources (Global)

- **Weekly Purification** (Tylobic): gold and silver for 100 / 200 / 300 points of Kurzan and Chaos Dungeon purification, by ilvl (1720+: 226,000 and 1,510,000). The gold figure looks too large for a weekly chest; unchecked, and the sheet says nothing of per character or bound.
- **Playtime Reward** (wiki): the client's six rows exactly; once a day per roster, live since 2026-04-15 with no end.
- **Login track**: no Global source lists its contents (an image only).
- **Waymark Mission** (wiki; the user called it Una's weekly): 100 points per weekly objective, 300 a week, each 100 buys one of four (chaos stone chest, soaring stone kit, 3 Lava or 9 Glacier breath), up to 1,000 banked. The wiki says per character; the client's tooltip says roster missions by the roster's highest character.
- **Arktoberfest** (2026-10-21 to 11-18): the wiki's shop matches the client's; weekly rewards need 770 tokens, one-time ones 650.
- **Guild shop**: the wiki has the cube ticket chests (1 a week per tier; 1640 2,320 bloodstones, 2,090 with the research). No source lists free weekly items.

## Paradise (Global, community)

- Hell: each descent 1–20 floors uniformly; Rare / Epic / Legendary keys 5 / 6 / 7 descents. Season 4 falling point every 10 floors: success (Maxroll 63%) a grand descent of 15–20, failure 1–5 back (Inven Global). Transmutation altar 20% on entry, guaranteed on 50 and 100.
- **No community-sampled floor distribution exists.** Vortex's floor-100 odds (0.0019% / 0.46% / 5.10% for 5 / 6 / 7 descents) are close to plain uniform descents (0.00003% / 0.36% / 4.43%), so likely computed.
- Weekly keys come from Crucible placement (season 1's official table: 3 Rare for everyone who clears, up to 6 for rank 1); season 4's counts are not published.
- No 1770 key tier anywhere (matches the client).

## Cube and Hourglass tiers

- Character to tier (wiki): 1640 and 1660 cube unlock 1, 1680 unlock 2, 1700 unlock 3, 1720 unlock 4, 1730 cube 4 and hourglass 1, 1750 hourglass 2, 1770 hourglass 3 (live in Global since 2026-10-21). Hourglass once a week per character, no ticket. Cube tickets 1–2 a week from Chaos Dungeon and Kurzan (wiki), plus a guild shop chest a week per tier.
- **Cube gems disagree**: the wiki has twice gcalc's and LoaGuard's (cube 1: 18 lvl-1 gem chests, two lvl-3 combined, against 9 lvl-1). Hourglass 1 and 2 agree (45, 54 in lvl-1), as do leapstones, breath and silver. Hourglass 3 is LoaGuard's only.

## Prices behind the tables

- **gcalc, KR, 2026-10-25 (10-26 where it changed):**
  - Crystals and leapstones: destruction crystal 8.3, guardian crystal 1.7, great leapstone 22 (21).
  - T4: destruction stone 1.6, guardian stone 0.3, leapstone 3.0.
  - Shards 0.098; silver 0.007.
  - Gems: lvl-1 gem 144 (146), hero gem 26,800, rare gem 98.3, common gem 3.5.
  - Breath: lava 469 (455), glacier 414 (407).
  - Card XP 1.2.
  - Exchange rate: 6.4–6.7 gold per KRW (Dicodico).
- **Tylobic, EUC, 2026-10-22, gold per unit, T4.1 / T4:**
  - Stones: red 20.48 / 3.99, blue 4.96 / 0.76.
  - Leapstone 17 / 10; shards 0.074; fusion 142 / 103.
  - Breath: glacier 417, lava 394.
  - Leapstones: circulated 125 (estimate), transferred 400 (estimate).
  - Lvl-1 T4 gem 198.4.
  - Content items: cube ticket (1720) 19,995; Haal's Hourglass (1750) 20,652; Chaos Gate (1770) 16,186; Sevek Atun (1770) 43,805.
  - Crystals: blue crystal 316.8; royal crystal 117.7.
  - Sheet totals check out as quantity × these prices.
- **loagg (Open API):** 운명의 돌파석 5, great leapstone 25, Abidos fusion 198, superior Abidos 269, circulated 100, transferred 250.

## How each values a run

- **Rested:** everyone takes it as 2x. Tylobic and Lost Mokokos double outright; LOAGAP says "estimated"; LoaGuard's 1770 guardian note says resting adds the same count again. The only measured hint against exact doubling is old Steam anecdote. loagg `/todo` calls rested play "66% 효율" (two runs every three days).
- **Bound materials:** valued at the tradable market price in every tool, shown in a separate column (gcalc "귀속", LoaGuard "귀속 골드", LOAGAP "거래불가"). loagg says bound materials are valued at "refining opportunity cost" but gives no formula. LoaGuard alone can subtract what the character already owns, and on the hell page it drops goods the character no longer needs.
- **Random drops:** expected values. Gems are the drop count times the official odds over the six kinds; lvl-1 gems are priced at (lvl-5 겁화 + lvl-5 작열) / 162. Sand and cube tickets are worth the content they open. Accessories, relic books and 편린 are left out by gcalc and LoaGuard as too variable.
- **Silver:** Tylobic leaves it out; gcalc uses 0.007 g; LoaGuard uses an unexplained reference table.
- **Card XP:** gcalc includes it at 1.2 g, LoaGuard has it off by default, and Tylobic does not list it.

## Cross-check observations

- **Shared Korean source:** gcalc and LoaGuard agree on every quantity they share (rift, Kurzan Front 1700–1720, guardian 1700–1750, cube, hourglass), so they share one upstream, not found. Neither gives a sample size.
- **KR against Tylobic:** within about 10% for rift 1730/1750, Kurzan Front and guardian silver. Rift 1730 destruction is equal (313). Rift 1770 differs most (420 / 1,200 / 54,000 against 469 / 1,422 / 63,747); LoaGuard's 1770 numbers are round, so probably provisional.
- **Kurzan Front 1700:** destruction differs, 512 (KR) against 630 (Tylobic).
- **LOAGAP disagrees with everyone:** about half the stones and fewer gems. It looks like pre-2026-02-04 data, when Kurzan stones were still tradable.

## Open

- Rested rift multiplier: unmeasured.
- 편린 rate and outcomes in the rift: only an old Inven sample.
- The upstream of the KR tables: maybe namu.wiki (unreadable here) or an Inven sample post.
- Hell floor distribution: unsampled; simulate from the rules.
- Cube gem counts: wiki against gcalc / LoaGuard, a factor of 2.
- Login track contents, guild shop free items, Growth Support (solo) shop sources beyond the client.
