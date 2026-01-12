-- decided to add these directly to vaults instead of separate table since it's 1:1
-- and avoids extra joins on every auth check
ALTER TABLE public.vaults 
ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN DEFAULT FALSE,
-- using 32-char base32 secret for TOTP compatibility
ADD COLUMN IF NOT EXISTS mfa_secret VARCHAR(32),
-- backup codes stored as array for convenience, encrypted at app layer
ADD COLUMN IF NOT EXISTS mfa_backup_codes TEXT[];

-- separate table because this grows indefinitely and we don't want to bloat vaults
CREATE TABLE IF NOT EXISTS public.mfa_audit_log (
    id SERIAL PRIMARY KEY,
    vault_address VARCHAR(44) NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'enable', 'disable', 'verify_success', 'verify_failed'
    -- storing IP and user agent for security monitoring
    -- IPv6 can be up to 45 chars
    ip_address VARCHAR(45),
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- composite index because we always filter by vault first then sort by time
CREATE INDEX idx_mfa_audit_vault ON public.mfa_audit_log(vault_address, created_at DESC);
-- separate index for security team to check failed attempts across all vaults
CREATE INDEX idx_mfa_audit_action ON public.mfa_audit_log(action, created_at DESC);

COMMENT ON TABLE public.mfa_audit_log IS 'Audit log for MFA authentication attempts';
