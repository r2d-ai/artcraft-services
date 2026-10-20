// Self-host (internal) deployment mode.
//
// Internal deployments are unmetered: generation is free and there is no
// billing at all, so every pricing / plans / upgrade / credits-purchase UI is
// switched off. Driven by the same flag the deploy Docker build uses for the
// self-hosted API configuration (see main.tsx); the hosted app leaves it unset
// and keeps all billing UI visible.
export const SELF_HOST: boolean =
  import.meta.env.VITE_ARTCRAFT_SELF_HOSTED === "true";
