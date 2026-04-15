DELETE FROM authz.role_permission
WHERE role_key IN (
  'algorithm_reviewer','compute_environment_operator','federation_task_admin',
  'proof_service_admin','public_chain_admin','profitshare_admin',
  'payout_operator','channel_split_admin',
  'execution_boundary_admin','model_custody_admin',
  'model_user','federation_initiator','federation_participant_admin'
);
