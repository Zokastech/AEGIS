#!/usr/bin/env node
/**
 * Rasterise les SVG AEGIS (logos, badges, réseaux sociaux) en PNG / ICO / WebP.
 * Prérequis : npm install (dans aegis-branding/)
 */
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import pngToIco from "png-to-ico";
import sharp from "sharp";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..");

async function mkdirp(dir) {
  await fs.mkdir(dir, { recursive: true });
}

async function main() {
  const logoDir = path.join(root, "logo");
  const iconSvg = await fs.readFile(path.join(logoDir, "aegis-icon.svg"));
  const iconMonoSvg = await fs.readFile(path.join(logoDir, "aegis-icon-mono.svg"));
  const fullSvg = await fs.readFile(path.join(logoDir, "aegis-full.svg"));
  const sqSvg = await fs.readFile(path.join(logoDir, "aegis-lockup-square.svg"));

  const sizes = [16, 32, 64, 128, 256, 512];
  const ratioFull = 72 / 240;
  const ratioSq = 140 / 120;

  for (const s of sizes) {
    const dir = path.join(logoDir, "png", `${s}px`);
    await mkdirp(dir);
    await sharp(iconSvg)
      .resize(s, s, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
      .png()
      .toFile(path.join(dir, "aegis-icon.png"));
    await sharp(iconMonoSvg)
      .resize(s, s, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
      .png()
      .toFile(path.join(dir, "aegis-icon-mono.png"));
  }

  for (const s of [128, 256, 512]) {
    const dir = path.join(logoDir, "png", `${s}px`);
    await mkdirp(dir);
    const hf = Math.max(1, Math.round(s * ratioFull));
    await sharp(fullSvg).resize(s, hf).png().toFile(path.join(dir, "aegis-full.png"));
    const hs = Math.max(1, Math.round(s * ratioSq));
    await sharp(sqSvg).resize(s, hs).png().toFile(path.join(dir, "aegis-lockup-square.png"));
  }

  const icoBuffers = await Promise.all(
    [16, 32, 48].map((s) =>
      sharp(iconSvg)
        .resize(s, s, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
        .png()
        .toBuffer(),
    ),
  );
  await fs.writeFile(path.join(logoDir, "favicon.ico"), await pngToIco(icoBuffers));
  await sharp(iconSvg)
    .resize(48, 48, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .webp({ quality: 92 })
    .toFile(path.join(logoDir, "favicon.webp"));

  const badgeDir = path.join(root, "badges");
  await sharp(await fs.readFile(path.join(badgeDir, "powered-by.svg")))
    .resize(400, 112)
    .png()
    .toFile(path.join(badgeDir, "powered-by.png"));
  await sharp(await fs.readFile(path.join(badgeDir, "aegis-certified.svg")))
    .resize(440, 112)
    .png()
    .toFile(path.join(badgeDir, "aegis-certified.png"));

  const social = [
    ["social-media/github-banner.svg", "social-media/github-banner.png", 1280, 640],
    ["social-media/twitter-card.svg", "social-media/twitter-card.png", 1200, 675],
    ["social-media/linkedin-banner.svg", "social-media/linkedin-banner.png", 1500, 500],
    ["social-media/og-image.svg", "social-media/og-image.png", 1200, 630],
  ];
  for (const [relSvg, relPng, w, h] of social) {
    const buf = await fs.readFile(path.join(root, relSvg));
    await sharp(buf).resize(w, h).png().toFile(path.join(root, relPng));
  }

  console.log("AEGIS branding export OK — logo/png, favicons, badges, social-media");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
