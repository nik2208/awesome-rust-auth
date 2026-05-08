(() => {
  const fallbackConfig = {
    apiPrefix: "/auth",
    loginPath: "/auth/login",
    refreshPath: "/auth/refresh",
    logoutPath: "/auth/logout",
    mePath: "/auth/me",
    registerPath: "/auth/register",
    builtInLocales: ["en", "it"]
  };

  const translations = {
    en: {
      title: "Auth UI",
      subtitle: "Sign in with your email and password",
      email: "Email",
      password: "Password",
      login: "Login",
      loading: "Signing in...",
      invalidConfig: "Unable to load auth config; using defaults."
    },
    it: {
      title: "Interfaccia Auth",
      subtitle: "Accedi con email e password",
      email: "Email",
      password: "Password",
      login: "Accedi",
      loading: "Accesso in corso...",
      invalidConfig: "Impossibile caricare la configurazione auth; uso dei default."
    }
  };

  function getLocale(config) {
    const browserLocale = (navigator.language || "en").split("-")[0];
    return (config.builtInLocales || []).includes(browserLocale) ? browserLocale : "en";
  }

  function applyI18n(locale) {
    const dict = translations[locale] || translations.en;
    document.querySelectorAll("[data-i18n]").forEach((element) => {
      const key = element.getAttribute("data-i18n");
      if (key && dict[key]) {
        element.textContent = dict[key];
      }
    });
  }

  async function loadConfig() {
    try {
      const response = await fetch("/auth/ui/config", { credentials: "same-origin" });
      if (!response.ok) {
        return fallbackConfig;
      }
      return { ...fallbackConfig, ...(await response.json()) };
    } catch (_) {
      return fallbackConfig;
    }
  }

  async function request(path, payload) {
    const response = await fetch(path, {
      method: "POST",
      headers: { "content-type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload || {})
    });
    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("application/json")) {
      return response.json();
    }
    return { status: response.status };
  }

  async function init() {
    const config = await loadConfig();
    const locale = getLocale(config);
    applyI18n(locale);

    const notice = document.getElementById("auth-notice");
    if (notice && config === fallbackConfig) {
      notice.textContent = translations[locale].invalidConfig;
    }

    return { config, locale };
  }

  window.AwesomeRustAuth = {
    version: "0.1.0",
    init,
    login: async (payload) => request((await loadConfig()).loginPath, payload),
    register: async (payload) => request((await loadConfig()).registerPath, payload),
    refresh: async () => request((await loadConfig()).refreshPath, {}),
    logout: async () => request((await loadConfig()).logoutPath, {}),
    me: async () => {
      const config = await loadConfig();
      const response = await fetch(config.mePath, { credentials: "include" });
      const contentType = response.headers.get("content-type") || "";
      if (contentType.includes("application/json")) {
        return response.json();
      }
      return { status: response.status };
    }
  };
})();
