CREATE TYPE transaction_type AS ENUM ('deposit', 'withdrawal', 'lock', 'unlock', 'transfer');

CREATE TABLE vaults (
    id SERIAL PRIMARY KEY,
    owner VARCHAR(44) NOT NULL UNIQUE,
    vault_address VARCHAR(44) NOT NULL UNIQUE,
    total_balance BIGINT NOT NULL DEFAULT 0,
    locked_balance BIGINT NOT NULL DEFAULT 0,
    available_balance BIGINT NOT NULL DEFAULT 0,
    total_deposited BIGINT NOT NULL DEFAULT 0,
    total_withdrawn BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT positive_balances CHECK (
        total_balance >= 0 AND
        locked_balance >= 0 AND
        available_balance >= 0 AND
        total_deposited >= 0 AND
        total_withdrawn >= 0
    ),
    CONSTRAINT balance_consistency CHECK (
        total_balance = locked_balance + available_balance
    )
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    vault_address VARCHAR(44) NOT NULL,
    transaction_type transaction_type NOT NULL,
    amount BIGINT NOT NULL,
    signature VARCHAR(88) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_vault
        FOREIGN KEY (vault_address)
        REFERENCES vaults(vault_address)
        ON DELETE CASCADE,
    CONSTRAINT positive_amount CHECK (amount > 0)
);

ALTER TABLE transactions
    ADD CONSTRAINT transactions_signature_unique UNIQUE (signature);

CREATE TABLE balance_snapshots (
    id SERIAL PRIMARY KEY,
    vault_address VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    snapshot_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_vault_snapshot
        FOREIGN KEY (vault_address)
        REFERENCES vaults(vault_address)
        ON DELETE CASCADE
);

CREATE INDEX idx_vaults_owner ON vaults(owner);
CREATE INDEX idx_vaults_vault_address ON vaults(vault_address);
CREATE INDEX idx_transactions_vault_address ON transactions(vault_address);
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_type ON transactions(transaction_type);
CREATE INDEX idx_balance_snapshots_vault_address ON balance_snapshots(vault_address);
CREATE INDEX idx_balance_snapshots_snapshot_time ON balance_snapshots(snapshot_time DESC);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_vaults_updated_at
    BEFORE UPDATE ON vaults
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE FUNCTION take_balance_snapshot()
RETURNS void AS $$
BEGIN
    INSERT INTO balance_snapshots (vault_address, total_balance, locked_balance, available_balance)
    SELECT vault_address, total_balance, locked_balance, available_balance
    FROM vaults;
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE vaults IS 'Stores vault state for each user';
COMMENT ON TABLE transactions IS 'Records all vault transactions';
COMMENT ON TABLE balance_snapshots IS 'Historical balance data for analytics';
COMMENT ON COLUMN vaults.total_balance IS 'Total USDT balance in lamports';
COMMENT ON COLUMN vaults.locked_balance IS 'Balance locked for open positions';
COMMENT ON COLUMN vaults.available_balance IS 'Balance available for withdrawal';

