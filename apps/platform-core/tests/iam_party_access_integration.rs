use db::{GenericClient, NoTls, connect};
use reqwest::Client;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn base_url() -> String {
    std::env::var("IAM_IT_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:18080".to_string())
}

fn database_url() -> String {
    std::env::var("IAM_IT_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://datab:datab_local_pass@127.0.0.1:5432/datab".to_string())
}

#[derive(Debug, Clone)]
struct LiveRunIds {
    suffix: String,
    org_name: String,
    invite_email: String,
    department_name: String,
    app_name: String,
    client_id: String,
    login_id: String,
    display_name: String,
    connector_name: String,
    environment_name: String,
    device_fingerprint_hash: String,
}

impl LiveRunIds {
    fn new() -> Self {
        let suffix = unique_suffix();
        Self {
            org_name: format!("IT IAM Org {suffix}"),
            invite_email: format!("it.invite+{suffix}@luna.local"),
            department_name: format!("IT Dept {suffix}"),
            app_name: format!("it-app-{suffix}"),
            client_id: format!("it-app-client-{suffix}"),
            login_id: format!("it.iam.user.{suffix}"),
            display_name: format!("IT IAM User {suffix}"),
            connector_name: format!("it-connector-{suffix}"),
            environment_name: format!("it-env-{suffix}"),
            device_fingerprint_hash: format!("it-fingerprint-{suffix}"),
            suffix,
        }
    }

    fn request_id(&self, step: &str) -> String {
        format!("it-iam-{step}-{}", self.suffix)
    }
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift before unix epoch")
        .as_nanos();
    format!("{:x}-{:x}", std::process::id(), now)
}

#[tokio::test]
#[ignore = "requires running platform-core service and local postgres"]
async fn iam_party_access_flow_live() {
    let client = Client::new();
    let base = base_url();
    let ids = LiveRunIds::new();

    let org = client
        .post(format!("{base}/api/v1/orgs/register"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .header("x-request-id", ids.request_id("org"))
        .body(format!(
            r#"{{"org_name":"{}","org_type":"enterprise"}}"#,
            ids.org_name
        ))
        .send()
        .await
        .expect("org register request");
    assert!(org.status().is_success(), "org status={}", org.status());
    let org_json: Value = org.json().await.expect("org json");
    let org_id = org_json["data"]["org_id"]
        .as_str()
        .expect("org_id")
        .to_string();
    let org_detail = client
        .get(format!("{base}/api/v1/iam/orgs/{org_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("org detail request");
    assert!(
        org_detail.status().is_success(),
        "org detail status={}",
        org_detail.status()
    );
    let org_list = client
        .get(format!("{base}/api/v1/iam/orgs"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("org list request");
    assert!(
        org_list.status().is_success(),
        "org list status={}",
        org_list.status()
    );

    let invite = client
        .post(format!("{base}/api/v1/iam/invitations"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .header("x-request-id", ids.request_id("invite"))
        .body(format!(
            r#"{{"org_id":"{org_id}","invited_email":"{}"}}"#,
            ids.invite_email
        ))
        .send()
        .await
        .expect("invite request");
    assert!(
        invite.status().is_success(),
        "invite status={}",
        invite.status()
    );

    let dept = client
        .post(format!("{base}/api/v1/iam/departments"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(format!(
            r#"{{"org_id":"{org_id}","department_name":"{}"}}"#,
            ids.department_name
        ))
        .send()
        .await
        .expect("department create request");
    assert!(
        dept.status().is_success(),
        "department status={}",
        dept.status()
    );
    let dept_json: Value = dept.json().await.expect("dept json");
    let dept_id = dept_json["data"]["department_id"]
        .as_str()
        .expect("department_id")
        .to_string();
    let dept_detail = client
        .get(format!("{base}/api/v1/iam/departments/{dept_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("department detail request");
    assert!(
        dept_detail.status().is_success(),
        "department detail status={}",
        dept_detail.status()
    );
    let dept_list = client
        .get(format!("{base}/api/v1/iam/departments?org_id={org_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("department list request");
    assert!(
        dept_list.status().is_success(),
        "department list status={}",
        dept_list.status()
    );

    let app = client
        .post(format!("{base}/api/v1/apps"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .header("x-request-id", ids.request_id("app"))
        .body(format!(
            r#"{{"org_id":"{org_id}","app_name":"{}","client_id":"{}"}}"#,
            ids.app_name, ids.client_id
        ))
        .send()
        .await
        .expect("app create request");
    assert!(app.status().is_success(), "app status={}", app.status());
    let app_json: Value = app.json().await.expect("app json");
    let app_id = app_json["data"]["app_id"]
        .as_str()
        .expect("app_id")
        .to_string();

    let forbidden = client
        .patch(format!("{base}/api/v1/apps/{app_id}"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_operator")
        .header("x-request-id", ids.request_id("forbidden"))
        .body(r#"{"status":"disabled"}"#)
        .send()
        .await
        .expect("forbidden request");
    assert_eq!(
        forbidden.status(),
        403,
        "forbidden status={}",
        forbidden.status()
    );

    let user = client
        .post(format!("{base}/api/v1/iam/users"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .header("x-request-id", ids.request_id("user"))
        .body(format!(
            r#"{{"org_id":"{org_id}","login_id":"{}","display_name":"{}"}}"#,
            ids.login_id, ids.display_name
        ))
        .send()
        .await
        .expect("user create request");
    assert!(user.status().is_success(), "user status={}", user.status());
    let user_json: Value = user.json().await.expect("user json");
    let user_id = user_json["data"]["user_id"]
        .as_str()
        .expect("user_id")
        .to_string();
    let user_detail = client
        .get(format!("{base}/api/v1/iam/users/{user_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("user detail request");
    assert!(
        user_detail.status().is_success(),
        "user detail status={}",
        user_detail.status()
    );
    let user_list = client
        .get(format!("{base}/api/v1/iam/users?org_id={org_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("user list request");
    assert!(
        user_list.status().is_success(),
        "user list status={}",
        user_list.status()
    );

    let login = client
        .post(format!("{base}/api/v1/auth/login"))
        .header("content-type", "application/json")
        .header("x-role", "platform_admin")
        .header("x-request-id", ids.request_id("login"))
        .body(format!(r#"{{"login_id":"{}"}}"#, ids.login_id))
        .send()
        .await
        .expect("login request");
    assert!(
        login.status().is_success(),
        "login status={}",
        login.status()
    );
    let login_json: Value = login.json().await.expect("login json");
    let session_id = login_json["data"]["session_id"]
        .as_str()
        .expect("session_id")
        .to_string();

    let revoke_session = client
        .post(format!("{base}/api/v1/iam/sessions/{session_id}/revoke"))
        .header("x-role", "tenant_admin")
        .header("x-request-id", "it-iam-session-revoke")
        .send()
        .await
        .expect("session revoke request");
    assert!(
        revoke_session.status().is_success(),
        "session revoke status={}",
        revoke_session.status()
    );

    let (pg, conn) = connect(&database_url(), NoTls).await.expect("db connect");
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let row = pg
        .query_one(
            "INSERT INTO iam.trusted_device (
               user_id, device_fingerprint_hash, device_name, platform, browser, trust_level, status
             ) VALUES (
               $1::text::uuid, $2, 'it-device', 'linux', 'firefox', 'trusted', 'active'
             )
             RETURNING trusted_device_id::text",
            &[&user_id, &ids.device_fingerprint_hash],
        )
        .await
        .expect("insert device");
    let device_id: String = row.get(0);

    let revoke_device = client
        .post(format!("{base}/api/v1/iam/devices/{device_id}/revoke"))
        .header("x-role", "tenant_admin")
        .header("x-request-id", ids.request_id("device-revoke"))
        .send()
        .await
        .expect("device revoke request");
    assert!(
        revoke_device.status().is_success(),
        "device revoke status={}",
        revoke_device.status()
    );

    let app_list = client
        .get(format!("{base}/api/v1/apps?org_id={org_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("app list request");
    assert!(
        app_list.status().is_success(),
        "app list status={}",
        app_list.status()
    );

    let connector = client
        .post(format!("{base}/api/v1/iam/connectors"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(format!(
            r#"{{"org_id":"{org_id}","connector_name":"{}","connector_type":"api","endpoint_ref":"https://example.local"}}"#,
            ids.connector_name
        ))
        .send()
        .await
        .expect("connector create request");
    assert!(
        connector.status().is_success(),
        "connector status={}",
        connector.status()
    );
    let connector_json: Value = connector.json().await.expect("connector json");
    let connector_id = connector_json["data"]["connector_id"]
        .as_str()
        .expect("connector_id")
        .to_string();
    let connector_detail = client
        .get(format!("{base}/api/v1/iam/connectors/{connector_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("connector detail request");
    assert!(
        connector_detail.status().is_success(),
        "connector detail status={}",
        connector_detail.status()
    );
    let connector_list = client
        .get(format!("{base}/api/v1/iam/connectors?org_id={org_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("connector list request");
    assert!(
        connector_list.status().is_success(),
        "connector list status={}",
        connector_list.status()
    );

    let env = client
        .post(format!("{base}/api/v1/iam/execution-environments"))
        .header("content-type", "application/json")
        .header("x-role", "tenant_admin")
        .body(format!(
            r#"{{"org_id":"{org_id}","connector_id":"{connector_id}","environment_name":"{}","environment_type":"sandbox","region_code":"cn-shanghai"}}"#,
            ids.environment_name
        ))
        .send()
        .await
        .expect("execution environment create request");
    assert!(env.status().is_success(), "env status={}", env.status());
    let env_json: Value = env.json().await.expect("env json");
    let env_id = env_json["data"]["environment_id"]
        .as_str()
        .expect("environment_id")
        .to_string();
    let env_detail = client
        .get(format!("{base}/api/v1/iam/execution-environments/{env_id}"))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("execution environment detail request");
    assert!(
        env_detail.status().is_success(),
        "env detail status={}",
        env_detail.status()
    );
    let env_list = client
        .get(format!(
            "{base}/api/v1/iam/execution-environments?org_id={org_id}"
        ))
        .header("x-role", "tenant_admin")
        .send()
        .await
        .expect("execution environment list request");
    assert!(
        env_list.status().is_success(),
        "env list status={}",
        env_list.status()
    );

    let logout = client
        .post(format!("{base}/api/v1/auth/logout"))
        .header("content-type", "application/json")
        .header("x-role", "platform_admin")
        .header("x-request-id", ids.request_id("logout"))
        .body(format!(r#"{{"session_id":"{session_id}"}}"#))
        .send()
        .await
        .expect("logout request");
    assert!(
        logout.status().is_success(),
        "logout status={}",
        logout.status()
    );
}
