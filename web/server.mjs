import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { dirname, join, normalize, sep } from "node:path";
import { fileURLToPath } from "node:url";

const port = Number(process.env.PORT ?? 4173);
const root = dirname(fileURLToPath(import.meta.url));

createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", `http://${req.headers.host ?? "localhost"}`);
  const file = url.pathname === "/" ? "index.html" : url.pathname.slice(1);
  const target = normalize(join(root, decodeURIComponent(file)));

  try {
    if (target !== root && !target.startsWith(`${root}${sep}`)) {
      throw new Error("Invalid path");
    }
    const body = await readFile(target);
    const type = file.endsWith(".css")
      ? "text/css"
      : file.endsWith(".js")
        ? "application/javascript"
        : file.endsWith(".png")
          ? "image/png"
          : "text/html";
    res.writeHead(200, {
      "content-type": type.startsWith("image/") ? type : `${type}; charset=utf-8`
    });
    res.end(body);
  } catch {
    res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    res.end("Not found");
  }
}).listen(port, () => {
  console.log(`Arbiter proof viewer listening on http://localhost:${port}`);
});
