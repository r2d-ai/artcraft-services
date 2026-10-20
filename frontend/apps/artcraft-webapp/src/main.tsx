import "./lib/crypto-shims";
import { StrictMode } from "react";
import { BrowserRouter } from "react-router-dom";
import * as ReactDOM from "react-dom/client";
import { GoogleOAuthProvider } from "@react-oauth/google";
import App from "./app/app";
import { StorytellerApiHostStore, UsersApi } from "@storyteller/api";
import { captureLandingContext, getReferrer } from "@storyteller/common";
import { setOmniGenErrorNotifier } from "@storyteller/omni-gen";
import { setToastDelegate } from "@storyteller/ui-toaster";
import { toast } from "./components/toast/toast";
import { captureLoginBridgeContext, LOGIN_BRIDGE_PATH } from "./lib/login-bridge-context";

captureLoginBridgeContext();
const isLoginBridge = window.location.pathname === LOGIN_BRIDGE_PATH;

const GOOGLE_CLIENT_ID = import.meta.env.VITE_GOOGLE_CLIENT_ID;

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement,
);

// Self-hosted production: API on same origin (nginx proxies /v1 to artcraft-api).
if (
  !import.meta.env.DEV &&
  import.meta.env.VITE_ARTCRAFT_SELF_HOSTED === "true"
) {
  try {
    StorytellerApiHostStore.getInstance().configureSelfHostedProduction();
  } catch (e) {
    console.warn("Failed to configure self-hosted API host", e);
  }
}

// In development, route API through the Vite dev server origin to avoid CORS
if (import.meta.env.DEV) {
  try {
    StorytellerApiHostStore.getInstance().setApiSchemeAndHost(
      window.location.origin,
    );
    // In dev we always route API through the Vite dev server origin so the
    // browser sends same-origin requests (cookies included) and Vite's /v1
    // proxy forwards them to the backend (http://k7:12345). Do not override
    // the API host via VITE_USE_LOCAL_API here — that makes the browser call
    // the backend directly and breaks cookies/CORS.
  } catch (e) {
    console.warn("Failed to set dev API host override", e);
  }
}

// Surface omni model/generation outages through this app's toast component.
setOmniGenErrorNotifier((message) => toast.error(message));

// Shared libs (promptbox deck, gallery modal, …) fire react-hot-toast toasts,
// but this app renders its own ToastContainer and never mounts the
// react-hot-toast container — route those toasts here so limit/validation
// errors (e.g. "audio too long") actually show up.
setToastDelegate({
  success: (message) => toast.success(message),
  error: (message) => toast.error(message),
});

// Persist landing context (referral username, landing URL, referrer) to apex-
// domain cookies so attribution survives the getartcraft.com →
// app.getartcraft.com hop. First visit wins.
if (!isLoginBridge) captureLandingContext();

// Fire-and-forget: log the referral once per browser session
if (!isLoginBridge && !sessionStorage.getItem("referral_logged")) {
  sessionStorage.setItem("referral_logged", "1");
  const referrer = getReferrer();
  new UsersApi()
    .LogWebReferral({ maybeReferralUrl: referrer })
    .then(() => {
      console.log("maybeReferralUrl", referrer);
    })
    .catch(() => {});
}

root.render(
  <StrictMode>
    <GoogleOAuthProvider clientId={GOOGLE_CLIENT_ID}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </GoogleOAuthProvider>
  </StrictMode>,
);
