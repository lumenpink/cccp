export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const ua = (request.headers.get("user-agent") || "").toLowerCase();
    const isCli = ua.includes("curl") || ua.includes("wget") || ua.includes("httpie");

    // If requested by CLI at root / or /install, serve installer
    if (url.pathname === "/install" || url.pathname === "/install.sh" || (url.pathname === "/" && isCli)) {
      const installReq = new Request(new URL("/install.sh", request.url), request);
      const res = await env.ASSETS.fetch(installReq);
      if (res.status === 200) {
        return new Response(res.body, {
          status: 200,
          headers: {
            "content-type": "text/x-shellscript; charset=utf-8",
            "cache-control": "public, max-age=300"
          }
        });
      }
    }

    // Otherwise serve static assets normally
    return env.ASSETS.fetch(request);
  }
};
