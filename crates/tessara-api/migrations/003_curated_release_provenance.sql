ALTER TABLE module_releases
    DROP CONSTRAINT module_releases_trust_state_check;

ALTER TABLE module_releases
    ADD CONSTRAINT module_releases_trust_state_check
    CHECK (trust_state IN ('unknown', 'curated', 'trusted', 'rejected'));

UPDATE module_releases
SET trust_state = 'curated'
WHERE trust_state = 'trusted';

UPDATE deployment_receipts
SET receipt = receipt - 'verified_publishers'
WHERE receipt ? 'verified_publishers';
