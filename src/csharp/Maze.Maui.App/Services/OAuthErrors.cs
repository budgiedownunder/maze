namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Thrown when the OAuth callback URL carries a `reason=` query parameter
    /// instead of a `token`/`expires_at` pair. The server emits these for the
    /// recoverable error paths (signup disabled, email not verified, user
    /// cancelled at the provider, etc.) so the client can show a friendly
    /// message rather than a generic failure.
    /// </summary>
    public class OAuthFlowFailedException : Exception
    {
        /// <summary>Raw error code from the server, e.g. "signup_disabled",
        /// "email_not_verified", or "provider_error:access_denied".</summary>
        public string Reason { get; }

        public OAuthFlowFailedException(string reason)
            : base($"OAuth flow failed: {reason}")
        {
            Reason = reason;
        }
    }

    /// <summary>
    /// Maps an OAuth error code (the value of the `reason` query parameter on
    /// the callback URL, or the `?error=…` value on a follow-up redirect) to
    /// a friendly user-facing message. Mirrors the React-side
    /// <c>getOAuthErrorMessage</c> helper in <c>src/utils/oauth.ts</c> so both
    /// clients show consistent text. Unknown codes return a generic fallback
    /// rather than echoing the raw code into the UI.
    /// </summary>
    public static class OAuthErrorMessages
    {
        public static string FromReason(string? code)
        {
            if (string.IsNullOrEmpty(code))
                return "We could not sign you in. Please try again.";

            // Provider-side errors arrive as `provider_error:<provider-code>`
            // (e.g. `provider_error:access_denied` when the user declines
            // consent at the provider).
            if (code.StartsWith("provider_error:", StringComparison.Ordinal))
            {
                var detail = code["provider_error:".Length..];
                return detail == "access_denied"
                    ? "Sign-in was cancelled at the provider."
                    : $"The provider reported an error ({detail}). Please try again.";
            }

            return code switch
            {
                "signup_disabled" =>
                    "Sign-up is disabled on this server. Sign-in via this provider is only available to existing users.",
                "email_not_verified" =>
                    "The provider did not return a verified email address. Please verify your email with the provider and try again.",
                "missing_email" =>
                    "The provider did not return an email address, so we cannot complete sign-in.",
                "invalid_state" or "missing_state" or "state_mismatch" or "state_expired" or "provider_mismatch" =>
                    "The sign-in session expired or was invalid. Please try again.",
                "missing_code" or "provider_response" =>
                    "There was a problem completing sign-in with the provider. Please try again.",
                "store_error" =>
                    "A server error occurred while completing sign-in. Please try again later.",
                _ => "We could not sign you in. Please try again.",
            };
        }
    }
}
