-- Revert 'production' from the allowed values for subscription status
ALTER TABLE subscriptions DROP CHECK subscriptions_chk_2;
ALTER TABLE subscriptions
ADD CONSTRAINT subscriptions_chk_2 CHECK (
        status IN ('trial', 'active', 'suspended', 'cancelled')
    );