// Runs a batch of the optimizer lab's queued CPU work in order, at below normal priority. Ctrl-C
// stops; running it again goes on from the job that was stopped (each job resumes itself, as
// `arena run --resume` does). `pnpm lab [status] iteration2/surrogate/a` reads
// `iteration2/surrogate/runs/a_queue.json` and writes `a_done.json` and `a_run.log` beside it.
// See `AI notes/Calculator core/Optimizer lab.md`.
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const root = path.join(import.meta.dirname, "../..");
const status = process.argv[2] === "status";
const batch = process.argv[status ? 3 : 2];
if (!batch) {
  console.log("pnpm lab [status] <iteration>/<investigation>/<batch>");
  process.exit(1);
}
const stem = path.join(root, "arena_data/lab", path.dirname(batch), "runs", path.basename(batch));
const queue_path = `${stem}_queue.json`;
const done_path = `${stem}_done.json`;
const log_path = `${stem}_run.log`;
const read = (file, fallback) =>
  fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : fallback;
const log = (text) => {
  process.stdout.write(text);
  fs.appendFileSync(log_path, text);
};

// [{ name, cmd, note? }], run from the repo root; a name is done once it exits 0
// read again before each job, so jobs queued meanwhile are picked up
const next = () => read(queue_path, []).find((job) => !done[job.name]);
const done = read(done_path, {});

if (status) {
  for (const job of read(queue_path, []))
    console.log(`${done[job.name] ? "done " + done[job.name] : "     pending       "}  ${job.name}`);
  process.exit(0);
}

// children inherit it
os.setPriority(os.constants.priority.PRIORITY_BELOW_NORMAL);
let stopping = false;
process.on("SIGINT", () => {
  stopping = true;
  console.log("\nstopping once the running job has stopped...");
});

let ran = 0;
for (let job = next(); job; job = next(), ran++) {
  log(`\n== ${job.name}${job.note ? ` (${job.note})` : ""}\n${job.cmd}\n`);
  const code = await new Promise((resolve) => {
    const child = spawn(job.cmd, { shell: true, stdio: ["inherit", "pipe", "pipe"], cwd: root });
    child.stdout.on("data", log);
    child.stderr.on("data", log);
    child.on("close", resolve);
  });
  if (stopping) {
    console.log("stopped; pnpm lab goes on from here");
    process.exit(0);
  }
  if (code !== 0) {
    console.log(`${job.name} exited with ${code}; fix it and run pnpm lab again`);
    process.exit(1);
  }
  done[job.name] = new Date().toISOString().slice(0, 16);
  fs.writeFileSync(done_path, JSON.stringify(done, null, 2) + "\n");
}
console.log(ran ? "queue done" : "nothing queued");
