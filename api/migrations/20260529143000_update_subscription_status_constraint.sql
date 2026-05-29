-- Add 'production' to the allowed values for subscription status
ALTER TABLE subscriptions DROP CHECK subscriptions_chk_2;
ALTER TABLE subscriptions
ADD CONSTRAINT subscriptions_chk_2 CHECK (
        status IN (
            'trial',
            'production',
            'active',
            'suspended',
            'cancelled'
        )
    );