import { useEffect, useState } from "react";
import { StorytellerApiHostStore } from "@storyteller/api";

type AuthConfig = {
  success: boolean;
  oidc_enabled: boolean;
  oidc_login_path?: string | null;
  oidc_only?: boolean;
};

export function OidcLoginButton() {
  const [loginPath, setLoginPath] = useState<string | null>(null);

  useEffect(() => {
    const api = StorytellerApiHostStore.getInstance().getApiSchemeAndHost();
    fetch(`${api}/v1/auth/config`, { credentials: "include" })
      .then((response) => response.json())
      .then((config: AuthConfig) => {
        if (config.oidc_enabled && config.oidc_login_path) {
          setLoginPath(config.oidc_login_path);
        }
      })
      .catch(() => {});
  }, []);

  if (!loginPath) {
    return null;
  }

  const fullUrl = `${StorytellerApiHostStore.getInstance().getApiSchemeAndHost()}${loginPath}`;

  return (
    <a
      className="flex w-full items-center justify-center rounded-lg border border-white/20 bg-white/5 px-4 py-2 text-sm font-medium text-white hover:bg-white/10"
      href={fullUrl}
    >
      Sign in with SSO
    </a>
  );
}
