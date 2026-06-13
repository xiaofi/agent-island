import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const outDir = join(root, "out");
const publicDir = join(root, "public");
const silentVideo = join(outDir, "agent-island-promo-silent.mp4");
const audioFile = join(outDir, "agent-island-bed.wav");
const finalVideo = join(outDir, "agent-island-promo.mp4");

mkdirSync(outDir, { recursive: true });
mkdirSync(publicDir, { recursive: true });

const run = (command, args) => {
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
};

if (process.env.SKIP_VISUAL_RENDER === "1" && existsSync(silentVideo)) {
  console.log(`Using existing ${silentVideo}`);
} else {
  run("npx", [
    "remotion",
    "render",
    "src/index.ts",
    "AgentIslandPromo",
    silentVideo,
    "--codec=h264",
    "--pixel-format=yuv420p",
  ]);
}

run("npx", [
  "remotion",
  "ffmpeg",
  "-y",
  "-f",
  "lavfi",
  "-i",
  "sine=frequency=58:sample_rate=48000:duration=30",
  "-f",
  "lavfi",
  "-i",
  "sine=frequency=116:sample_rate=48000:duration=30",
  "-f",
  "lavfi",
  "-i",
  "sine=frequency=232:sample_rate=48000:duration=30",
  "-filter_complex",
  "[0:a]volume=0.035[a0];[1:a]volume=0.018[a1];[2:a]volume=0.009[a2];[a0][a1][a2]amix=inputs=3:duration=first[a]",
  "-map",
  "[a]",
  "-ar",
  "48000",
  "-ac",
  "2",
  audioFile,
]);

run("npx", [
  "remotion",
  "ffmpeg",
  "-y",
  "-i",
  silentVideo,
  "-i",
  audioFile,
  "-map",
  "0:v:0",
  "-map",
  "1:a:0",
  "-c:v",
  "copy",
  "-c:a",
  "aac",
  "-b:a",
  "128k",
  "-shortest",
  finalVideo,
]);

console.log(`Rendered ${finalVideo}`);
