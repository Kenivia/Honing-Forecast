// Runs the optimizer lab's queued CPU work in order, at below normal priority. Ctrl-C stops;
// running it again goes on from the job that was stopped (each job resumes itself, as
// `arena run --resume` does). `pnpm lab [status]`. See `AI notes/Calculator core/Optimizer lab.md`.
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const root = path.join(import.meta.dirname, "../..");
const dir = path.join(root, "arena_data/lab");
const queue_path = path.join(dir, "queue.json");
const done_path = path.join(dir, "done.json");
const read = (file, fallback) =>
  fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : fallback;

// [{ name, cmd, note? }], run from the repo root; a name is done once it exits 0
// read again before each job, so jobs queued meanwhile are picked up
const next = () => read(queue_path, []).find((job) => !done[job.name]);
const done = read(done_path, {});

if (process.argv[2] === "status") {
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
  console.log(`\n== ${job.name}${job.note ? ` (${job.note})` : ""}\n${job.cmd}`);
  const code = await new Promise((resolve) =>
    spawn(job.cmd, { shell: true, stdio: "inherit", cwd: root }).on("exit", resolve),
  );
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
