use regex::RegexSet;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Normalizes an incoming request URL by stripping query parameters and trimming slashes.
///
/// Example: `/api/v1/mergerSchemes?active=true` becomes `/api/v1/mergerSchemes`.
pub fn normalize_url(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// Route classification policy holding the whitelist and blacklist of URLs.
///
/// In Virat NestJS (`logger.interceptor.ts`), route checks are performed on every HTTP
/// request to determine whether:
/// 1. The incoming request body is encrypted and needs decryption.
/// 2. The outgoing response body must be encrypted.
/// 3. The route uses a default static encryption key or client-specific public key.
/// 4. The route should be logged to the audit log.
pub struct RouteSecurityPolicy {
    not_encrypted_requests: HashSet<&'static str>,
    not_encrypted_responses: HashSet<&'static str>,
    not_encrypted_webhooks: HashSet<&'static str>,
    not_stringify_responses: HashSet<&'static str>,
    exclude_from_logging: HashSet<&'static str>,
    default_encryption_strings: HashSet<&'static str>,
    default_encryption_regex: RegexSet,
}

impl RouteSecurityPolicy {
    /// Builds the route security policy matching Virat's configuration.
    pub fn init() -> Self {
        let not_encrypted_requests = HashSet::from([
            "/health",
            "/api/v1/bank/webhookUpdateFundAccountValidation",
            "/api/v1/payment/webhook-update-payment",
            "/api/v1/payment/eMandateTokenConfirmedWebhook",
            "/api/v1/payment/eMandateTokenStoreWebhook",
            "/api/v1/dematrequest/saveDematDetailsCallback",
            "/api/v1/bharat-bond/cdslwebhook",
            "/api/v1/auth/bot/generate-otp",
            "/api/v1/auth/bot/verify-otp",
            "/api/v1/auth/mf/bot/otp/verify",
            "/api/v1/auth/mf/bot/otp/generate",
            "/api/v1/third-party/bot/report/distributorOrInvestorStatements",
            "/api/v1/third-party/bot/glx_getAnnexureDetail",
            "/api/v1/third-party/bot/glx_getPayoutDetails",
            "/api/v1/third-party/bot/glx_gstInvoice",
            "/api/v1/payment/eMandateUPITokenStoreWebhook",
            "/api/v1/mergerSchemes",
            "/api/v1/kfintech/encrypt",
            "/api/v1/kfintech/decrypt",
            "/api/v1/auth/encryption-key",
        ]);

        let not_encrypted_responses = HashSet::from([
            "/health",
            "/api/v1/auth/bot/generate-otp",
            "/api/v1/auth/bot/verify-otp",
            "/api/v1/auth/mf/bot/otp/verify",
            "/api/v1/auth/mf/bot/otp/generate",
            "/api/v1/third-party/bot/report/distributorOrInvestorStatements",
            "/api/v1/third-party/bot/glx_getAnnexureDetail",
            "/api/v1/third-party/bot/glx_getPayoutDetails",
            "/api/v1/bank/webhookUpdateFundAccountValidation",
            "/api/v1/payment/webhook-update-payment",
            "/api/v1/payment/eMandateTokenConfirmedWebhook",
            "/api/v1/dematrequest/saveDematDetailsCallback",
            "/api/v1/third-party/bot/glx_gstInvoice",
            "/api/v1/payment/webhook-update-payment",
            "/api/v1/payment/eMandateTokenStoreWebhook",
            "/api/v1/payment/eMandateUPITokenStoreWebhook",
            "/api/v1/kfintech/test",
            "/api/v1/kfintech/encrypt",
            "/api/v1/kfintech/decrypt",
            "/api/v1/mergerSchemes",
            "/api/v1/auth/encryption-key",
        ]);

        let not_encrypted_webhooks = HashSet::from([
            "/api/v1/payment/eMandateTokenConfirmedWebhook",
            "/api/v1/payment/webhook-update-payment",
            "/api/v1/payment/eMandateTokenStoreWebhook",
            "/api/v1/payment/eMandateUPITokenStoreWebhook",
            "/api/v1/kfintech/test",
            "/api/v1/kfintech/encrypt",
            "/api/v1/kfintech/decrypt",
        ]);

        let not_stringify_responses = HashSet::from([
            "/api/v1/third-party/bot/report/distributorOrInvestorStatements",
            "/api/v1/third-party/bot/glx_getPayoutDetails",
        ]);

        let exclude_from_logging = HashSet::from([
            "/api/v1/holder/account-opening-api-masters",
            "/api/v1/log",
            "/api/v1/bank/getBankDetailsByIFSC",
            "/api/v1/transaction/schemes",
        ]);

        let default_encryption_strings = HashSet::from([
            "/api/v1/master/config",
            "/api/v1/master",
            "/api/v1/master/market-content",
            "/api/v1/master/sif/scheme-details",
            "/api/v1/master/sif/account-opening-master",
            "/api/v1/nof-configuartion/config",
            "/api/v1/user/check-user-type",
            "/api/v1/user/CheckUserDobMobileEmail",
            "/api/v1/auth/generate-otp",
            "/api/v1/auth/verify-otp",
            "/api/v1/folio/checkKyc",
            "/api/v1/user/createNewFolioPanCheck",
            "/api/v1/auth/verify-folio-otp",
            "/api/v1/auth/session-otp",
            "/api/v1/auth/verify-captcha",
            "/api/v1/auth/generate-folio-otp",
            "/api/v1/folio",
            "/api/v1/folio/sif",
            "/api/v1/user/updateWhatsappConsent",
            "/api/v1/user/updatePIIConsent",
            "/api/v1/holder/account-opening-api-masters",
            "/api/v1/third-party/investor-account-report/checkDistributorValidation",
            "/api/v1/transaction/schemes",
            "/api/v1/transaction/schemeDetails",
            "/api/v1/distributorLink",
            "/api/v1/utm",
            "/api/v1/third-party/investor-account-report/getInvestorProfileDetails",
            "/api/v1/third-party/report/distributorOrInvestorStatements",
            "/api/v1/third-party/investor-account-report/getDistributorLoginDetails",
            "/api/v1/auth/dropOut/verify-otp",
            "/api/v1/auth/dropOut/generate-otp",
            "/api/v1/user/checkUserTypeAndFolioDetailAfterZBF",
            "/api/v1/bharat-bond/getBBMergerUserDataByPAN",
            "/api/v1/auth/verify-otp-bb",
            "/api/v1/bharat-bond/updateConsent",
            "/api/v1/bharat-bond/sendEmailTransaction",
            "/api/v1/auth/generate-otp-bb",
            "/api/v1/scheme/sif",
            "/api/v1/scheme/sif/details",
            "/api/v1/user/sif/check",
            "/api/v1/user/sif/validate",
            "/api/v1/user/checkPanAadharVaidation",
            "/api/v1/user/sif/get-user-emails",
            "/api/v1/report/sif",
            "/api/v1/report/sif/generate-without-login",
            "/api/v1/user/sif/check-distributor-folio",
            "/api/v1/user/sif/get-distributor-email",
            "/api/v1/scheme/sif/get-cycle-info-master",
            "/api/v1/scheme/get-cycle-info-master",
            "/api/v1/cart/mf/add",
            "/api/v1/cart/mf/count",
            "/api/v1/cart/mf",
            "/api/v1/bank/existing",
            "/api/v1/cart/mf/updateCart",
            "/api/v1/cart/mf/schemes/recommendation",
            "/api/v1/auth/logout",
            "/api/v1/distributor/sif/get-agent-details",
            "/api/v1/user/check-kyc-status",
            "/api/v1/user/check-pii-consent",
            "/api/v1/folio/createMinorFolio",
        ]);

        let default_encryption_regex = RegexSet::new([
            r"^/api/v1/master/features/\d+$",
            r"^/api/v1/cart/count/[a-fA-F0-9-]{36}$",
            r"^/api/v1/distributorLink/[a-fA-F0-9-]{36}$",
            r"^/api/v1/dropout/[a-fA-F0-9-]{36}$",
            r"^/api/v1/user/isWhatsappChecked/[A-Z0-9-]{10}$",
            r"^/api/v1/cart/mf/recommendate/.+$",
            r"^/api/v1/cart/mf/.+$",
            r"^/api/v1/third-party/GetArnLinkDataFromGalaxy/.+$",
        ])
        .expect("Failed to compile default encryption RegexSet");

        Self {
            not_encrypted_requests,
            not_encrypted_responses,
            not_encrypted_webhooks,
            not_stringify_responses,
            exclude_from_logging,
            default_encryption_strings,
            default_encryption_regex,
        }
    }

    /// Returns `true` if incoming request bodies for this URL are encrypted and require decryption.
    pub fn is_encrypted_request(&self, url: &str) -> bool {
        let path = normalize_url(url);
        !self.not_encrypted_requests.contains(path)
    }

    /// Returns `true` if outgoing responses for this URL must be encrypted before returning to client.
    pub fn is_encrypted_response(&self, url: &str) -> bool {
        let path = normalize_url(url);
        !self.not_encrypted_responses.contains(path)
    }

    /// Returns `true` if the URL is configured to use the default static AES encryption key.
    /// Matches either static string paths or dynamic regex patterns.
    pub fn is_default_encryption_url(&self, url: &str) -> bool {
        let path = normalize_url(url);
        if self.default_encryption_strings.contains(path) {
            return true;
        }
        self.default_encryption_regex.is_match(path)
    }

    /// Returns `true` if requests to this URL should be logged in the audit database.
    pub fn should_log(&self, url: &str) -> bool {
        let path = normalize_url(url);
        !self.exclude_from_logging.contains(path)
    }

    /// Returns `true` if this URL is a webhook that does not require response body stringification.
    pub fn is_not_encrypted_webhook(&self, url: &str) -> bool {
        let path = normalize_url(url);
        self.not_encrypted_webhooks.contains(path)
    }

    /// Returns `true` if this URL's response should not be stringified.
    pub fn is_not_stringify_response(&self, url: &str) -> bool {
        let path = normalize_url(url);
        self.not_stringify_responses.contains(path)
    }
}

/// Global thread-safe singleton for route security policies.
///
/// Initialized once upon first access, thread-safe across all Actix worker threads.
pub static ROUTE_POLICY: LazyLock<RouteSecurityPolicy> = LazyLock::new(RouteSecurityPolicy::init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("/api/v1/ping?test=1&user=abc"),
            "/api/v1/ping"
        );
        assert_eq!(normalize_url("/api/v1/ping"), "/api/v1/ping");
    }

    #[test]
    fn test_request_encryption_rules() {
        // Whitelisted unencrypted routes
        assert!(!ROUTE_POLICY.is_encrypted_request("/api/v1/mergerSchemes"));
        assert!(
            !ROUTE_POLICY.is_encrypted_request("/api/v1/payment/webhook-update-payment?foo=bar")
        );

        // Standard routes should be encrypted
        assert!(ROUTE_POLICY.is_encrypted_request("/api/v1/user/profile"));
        assert!(ROUTE_POLICY.is_encrypted_request("/api/v1/bank/create"));
    }

    #[test]
    fn test_response_encryption_rules() {
        // Whitelisted unencrypted responses
        assert!(!ROUTE_POLICY.is_encrypted_response("/api/v1/mergerSchemes"));
        assert!(!ROUTE_POLICY.is_encrypted_response("/api/v1/auth/bot/generate-otp"));

        // Standard responses should be encrypted
        assert!(ROUTE_POLICY.is_encrypted_response("/api/v1/user/profile"));
    }

    #[test]
    fn test_default_encryption_strings_and_regex() {
        // Static string match
        assert!(ROUTE_POLICY.is_default_encryption_url("/api/v1/master/config"));
        assert!(ROUTE_POLICY.is_default_encryption_url("/api/v1/auth/generate-otp"));

        // Dynamic regex pattern matches
        assert!(ROUTE_POLICY.is_default_encryption_url("/api/v1/master/features/42"));
        assert!(
            ROUTE_POLICY.is_default_encryption_url(
                "/api/v1/cart/count/12345678-1234-1234-1234-123456789abc"
            )
        );
        assert!(ROUTE_POLICY.is_default_encryption_url("/api/v1/cart/mf/recommendate/some-scheme"));

        // Non-matching routes
        assert!(!ROUTE_POLICY.is_default_encryption_url("/api/v1/payment/checkout"));
    }

    #[test]
    fn test_logging_exclusions() {
        // Excluded from logging
        assert!(!ROUTE_POLICY.should_log("/api/v1/log"));
        assert!(!ROUTE_POLICY.should_log("/api/v1/bank/getBankDetailsByIFSC"));

        // Included in logging
        assert!(ROUTE_POLICY.should_log("/api/v1/ping"));
        assert!(ROUTE_POLICY.should_log("/api/v1/mergerSchemes"));
    }
}
