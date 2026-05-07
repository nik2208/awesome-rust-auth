window.AwesomeRustAuth = {
  version: "0.1.0",
  login: async function (endpoint, payload) {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload)
    });
    return response.json();
  }
};
