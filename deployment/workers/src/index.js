const BOOTSTRAP_URL =
  "https://raw.githubusercontent.com/safreu/aims/main/deployment/bootstrap.sh";

export default {
  async fetch(request) {
    const url = new URL(request.url);

    if (url.pathname !== "/install") {
      return new Response("Not Found\n", {
        status: 404,
        headers: {
          "Content-Type": "text/plain; charset=utf-8",
        },
      });
    }

    const response = await fetch(BOOTSTRAP_URL);

    if (!response.ok) {
      return new Response("Failed to retrieve Aims installer.\n", {
        status: 502,
        headers: {
          "Content-Type": "text/plain; charset=utf-8",
        },
      });
    }

    return new Response(response.body, {
      status: 200,
      headers: {
        "Content-Type": "text/plain; charset=utf-8",
        "Cache-Control": "no-cache",
      },
    });
  },
};