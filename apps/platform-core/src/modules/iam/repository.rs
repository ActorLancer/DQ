use crate::modules::iam::domain::{
    ApplicationListQuery, ApplicationView, ConnectorListQuery, ConnectorView,
    CreateApplicationRequest, CreateConnectorRequest, CreateDepartmentRequest,
    CreateExecutionEnvironmentRequest, CreateUserRequest, DepartmentListQuery, DepartmentView,
    ExecutionEnvironmentListQuery, ExecutionEnvironmentView, PatchApplicationRequest,
    UserListQuery, UserView,
};
use db::{DbClientOps, DbRecord, Error};

pub struct PostgresIamRepository;

impl PostgresIamRepository {
    pub async fn create_department(
        client: &impl DbClientOps,
        payload: &CreateDepartmentRequest,
    ) -> Result<DepartmentView, Error> {
        let row = client
            .query_one(
                "INSERT INTO core.department (
                   org_id, department_name, parent_department_id
                 ) VALUES (
                   $1::text::uuid, $2, $3::text::uuid
                 )
                 RETURNING
                   department_id::text,
                   org_id::text,
                   department_name,
                   parent_department_id::text,
                   status",
                &[
                    &payload.org_id,
                    &payload.department_name,
                    &payload.parent_department_id,
                ],
            )
            .await?;
        Ok(DepartmentView {
            department_id: row.get(0),
            org_id: row.get(1),
            department_name: row.get(2),
            parent_department_id: row.get(3),
            status: row.get(4),
        })
    }

    pub async fn get_department(
        client: &impl DbClientOps,
        id: &str,
    ) -> Result<Option<DepartmentView>, Error> {
        let row = client
            .query_opt(
                "SELECT
                   department_id::text,
                   org_id::text,
                   department_name,
                   parent_department_id::text,
                   status
                 FROM core.department
                 WHERE department_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| DepartmentView {
            department_id: row.get(0),
            org_id: row.get(1),
            department_name: row.get(2),
            parent_department_id: row.get(3),
            status: row.get(4),
        }))
    }

    pub async fn list_departments(
        client: &impl DbClientOps,
        query: &DepartmentListQuery,
    ) -> Result<Vec<DepartmentView>, Error> {
        let rows = client
            .query(
                "SELECT department_id::text, org_id::text, department_name, parent_department_id::text, status
                 FROM core.department
                 WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
                   AND ($2::text IS NULL OR status = $2)
                 ORDER BY created_at DESC
                 LIMIT 100",
                &[&query.org_id, &query.status],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|row| DepartmentView {
                department_id: row.get(0),
                org_id: row.get(1),
                department_name: row.get(2),
                parent_department_id: row.get(3),
                status: row.get(4),
            })
            .collect())
    }

    pub async fn create_user(
        client: &impl DbClientOps,
        payload: &CreateUserRequest,
    ) -> Result<UserView, Error> {
        let row = client
            .query_one(
                "INSERT INTO core.user_account (
                   org_id, department_id, login_id, display_name, user_type, email, phone, mfa_status
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4, $5, $6, $7, 'pending'
                 )
                 RETURNING
                   user_id::text,
                   org_id::text,
                   department_id::text,
                   login_id::text,
                   display_name,
                   user_type,
                   status,
                   email::text,
                   phone",
                &[
                    &payload.org_id,
                    &payload.department_id,
                    &payload.login_id,
                    &payload.display_name,
                    &payload
                        .user_type
                        .clone()
                        .unwrap_or_else(|| "human".to_string()),
                    &payload.email,
                    &payload.phone,
                ],
            )
            .await?;
        Ok(UserView {
            user_id: row.get(0),
            org_id: row.get(1),
            department_id: row.get(2),
            login_id: row.get(3),
            display_name: row.get(4),
            user_type: row.get(5),
            status: row.get(6),
            email: row.get(7),
            phone: row.get(8),
        })
    }

    pub async fn get_user(client: &impl DbClientOps, id: &str) -> Result<Option<UserView>, Error> {
        let row = client
            .query_opt(
                "SELECT
                   user_id::text,
                   org_id::text,
                   department_id::text,
                   login_id::text,
                   display_name,
                   user_type,
                   status,
                   email::text,
                   phone
                 FROM core.user_account
                 WHERE user_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| UserView {
            user_id: row.get(0),
            org_id: row.get(1),
            department_id: row.get(2),
            login_id: row.get(3),
            display_name: row.get(4),
            user_type: row.get(5),
            status: row.get(6),
            email: row.get(7),
            phone: row.get(8),
        }))
    }

    pub async fn list_users(
        client: &impl DbClientOps,
        query: &UserListQuery,
    ) -> Result<Vec<UserView>, Error> {
        let rows = client
            .query(
                "SELECT user_id::text, org_id::text, department_id::text, login_id::text, display_name,
                        user_type, status, email::text, phone
                 FROM core.user_account
                 WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
                   AND ($2::text IS NULL OR department_id = $2::text::uuid)
                   AND ($3::text IS NULL OR status = $3)
                 ORDER BY created_at DESC
                 LIMIT 100",
                &[&query.org_id, &query.department_id, &query.status],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|row| UserView {
                user_id: row.get(0),
                org_id: row.get(1),
                department_id: row.get(2),
                login_id: row.get(3),
                display_name: row.get(4),
                user_type: row.get(5),
                status: row.get(6),
                email: row.get(7),
                phone: row.get(8),
            })
            .collect())
    }

    pub async fn create_app(
        client: &impl DbClientOps,
        payload: &CreateApplicationRequest,
    ) -> Result<ApplicationView, Error> {
        let row = client
            .query_one(
                "INSERT INTO core.application (
                   org_id, app_name, app_type, status, client_id, client_secret_hash, metadata
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'active', $4, $5, $6::jsonb
                 )
                 RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
                &[
                    &payload.org_id,
                    &payload.app_name,
                    &payload
                        .app_type
                        .clone()
                        .unwrap_or_else(|| "api_client".to_string()),
                    &payload.client_id,
                    &payload.client_secret_hash,
                    &serde_json::json!({
                        "client_secret_status": if payload.client_secret_hash.is_some() { "active" } else { "missing" }
                    }),
                ],
            )
            .await?;
        Ok(parse_app_row(&row))
    }

    pub async fn patch_app(
        client: &impl DbClientOps,
        id: &str,
        payload: &PatchApplicationRequest,
    ) -> Result<Option<ApplicationView>, Error> {
        let row = client
            .query_opt(
                "UPDATE core.application
                 SET
                   app_name = COALESCE($2, app_name),
                   status = COALESCE($3, status),
                   updated_at = now()
                 WHERE app_id = $1::text::uuid
                 RETURNING app_id::text, org_id::text, app_name, app_type, status, client_id, metadata",
                &[&id, &payload.app_name, &payload.status],
            )
            .await?;
        Ok(row.map(|row| parse_app_row(&row)))
    }

    pub async fn get_app(
        client: &impl DbClientOps,
        id: &str,
    ) -> Result<Option<ApplicationView>, Error> {
        let row = client
            .query_opt(
                "SELECT app_id::text, org_id::text, app_name, app_type, status, client_id, metadata
                 FROM core.application
                 WHERE app_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| parse_app_row(&row)))
    }

    pub async fn list_apps(
        client: &impl DbClientOps,
        query: &ApplicationListQuery,
    ) -> Result<Vec<ApplicationView>, Error> {
        let rows = client
            .query(
                "SELECT app_id::text, org_id::text, app_name, app_type, status, client_id, metadata
                 FROM core.application
                 WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
                   AND ($2::text IS NULL OR status = $2)
                 ORDER BY created_at DESC
                 LIMIT 100",
                &[&query.org_id, &query.status],
            )
            .await?;
        Ok(rows.iter().map(parse_app_row).collect())
    }

    pub async fn create_connector(
        client: &impl DbClientOps,
        payload: &CreateConnectorRequest,
    ) -> Result<ConnectorView, Error> {
        let row = client
            .query_one(
                "INSERT INTO core.connector (
                   org_id, connector_name, connector_type, status, endpoint_ref
                 ) VALUES (
                   $1::text::uuid, $2, $3, 'draft', $4
                 )
                 RETURNING connector_id::text, org_id::text, connector_name, connector_type, status, endpoint_ref",
                &[
                    &payload.org_id,
                    &payload.connector_name,
                    &payload.connector_type,
                    &payload.endpoint_ref,
                ],
            )
            .await?;
        Ok(ConnectorView {
            connector_id: row.get(0),
            org_id: row.get(1),
            connector_name: row.get(2),
            connector_type: row.get(3),
            status: row.get(4),
            endpoint_ref: row.get(5),
        })
    }

    pub async fn get_connector(
        client: &impl DbClientOps,
        id: &str,
    ) -> Result<Option<ConnectorView>, Error> {
        let row = client
            .query_opt(
                "SELECT connector_id::text, org_id::text, connector_name, connector_type, status, endpoint_ref
                 FROM core.connector
                 WHERE connector_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| ConnectorView {
            connector_id: row.get(0),
            org_id: row.get(1),
            connector_name: row.get(2),
            connector_type: row.get(3),
            status: row.get(4),
            endpoint_ref: row.get(5),
        }))
    }

    pub async fn list_connectors(
        client: &impl DbClientOps,
        query: &ConnectorListQuery,
    ) -> Result<Vec<ConnectorView>, Error> {
        let rows = client
            .query(
                "SELECT connector_id::text, org_id::text, connector_name, connector_type, status, endpoint_ref
                 FROM core.connector
                 WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
                   AND ($2::text IS NULL OR status = $2)
                 ORDER BY created_at DESC
                 LIMIT 100",
                &[&query.org_id, &query.status],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|row| ConnectorView {
                connector_id: row.get(0),
                org_id: row.get(1),
                connector_name: row.get(2),
                connector_type: row.get(3),
                status: row.get(4),
                endpoint_ref: row.get(5),
            })
            .collect())
    }

    pub async fn create_execution_environment(
        client: &impl DbClientOps,
        payload: &CreateExecutionEnvironmentRequest,
    ) -> Result<ExecutionEnvironmentView, Error> {
        let row = client
            .query_one(
                "INSERT INTO core.execution_environment (
                   org_id, connector_id, environment_name, environment_type, status, region_code
                 ) VALUES (
                   $1::text::uuid, $2::text::uuid, $3, $4, 'draft', $5
                 )
                 RETURNING environment_id::text, org_id::text, connector_id::text, environment_name, environment_type, status, region_code",
                &[
                    &payload.org_id,
                    &payload.connector_id,
                    &payload.environment_name,
                    &payload.environment_type,
                    &payload.region_code,
                ],
            )
            .await?;
        Ok(ExecutionEnvironmentView {
            environment_id: row.get(0),
            org_id: row.get(1),
            connector_id: row.get(2),
            environment_name: row.get(3),
            environment_type: row.get(4),
            status: row.get(5),
            region_code: row.get(6),
        })
    }

    pub async fn get_execution_environment(
        client: &impl DbClientOps,
        id: &str,
    ) -> Result<Option<ExecutionEnvironmentView>, Error> {
        let row = client
            .query_opt(
                "SELECT environment_id::text, org_id::text, connector_id::text, environment_name, environment_type, status, region_code
                 FROM core.execution_environment
                 WHERE environment_id = $1::text::uuid",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| ExecutionEnvironmentView {
            environment_id: row.get(0),
            org_id: row.get(1),
            connector_id: row.get(2),
            environment_name: row.get(3),
            environment_type: row.get(4),
            status: row.get(5),
            region_code: row.get(6),
        }))
    }

    pub async fn list_execution_environments(
        client: &impl DbClientOps,
        query: &ExecutionEnvironmentListQuery,
    ) -> Result<Vec<ExecutionEnvironmentView>, Error> {
        let rows = client
            .query(
                "SELECT environment_id::text, org_id::text, connector_id::text, environment_name, environment_type, status, region_code
                 FROM core.execution_environment
                 WHERE ($1::text IS NULL OR org_id = $1::text::uuid)
                   AND ($2::text IS NULL OR connector_id = $2::text::uuid)
                   AND ($3::text IS NULL OR status = $3)
                 ORDER BY created_at DESC
                 LIMIT 100",
                &[&query.org_id, &query.connector_id, &query.status],
            )
            .await?;
        Ok(rows
            .iter()
            .map(|row| ExecutionEnvironmentView {
                environment_id: row.get(0),
                org_id: row.get(1),
                connector_id: row.get(2),
                environment_name: row.get(3),
                environment_type: row.get(4),
                status: row.get(5),
                region_code: row.get(6),
            })
            .collect())
    }
}

fn parse_app_row(row: &DbRecord) -> ApplicationView {
    ApplicationView {
        app_id: row.get(0),
        org_id: row.get(1),
        app_name: row.get(2),
        app_type: row.get(3),
        status: row.get(4),
        client_id: row.get(5),
        client_secret_status: row
            .get::<_, serde_json::Value>(6)
            .get("client_secret_status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    }
}
