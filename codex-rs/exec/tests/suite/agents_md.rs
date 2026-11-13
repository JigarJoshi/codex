#![allow(clippy::unwrap_used)]

use codex_core::config::set_project_trust_level;
use codex_protocol::config_types::TrustLevel;
use core_test_support::responses;
use core_test_support::test_codex_exec::test_codex_exec;
use predicates::str::contains;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_includes_workspace_agents_md_in_request() -> anyhow::Result<()> {
    let test = test_codex_exec();
    std::fs::write(test.cwd_path().join("AGENTS.md"), "workspace instructions")?;

    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("resp1"),
        responses::ev_assistant_message("m1", "fixture hello"),
        responses::ev_completed("resp1"),
    ]);
    let response_mock = responses::mount_sse_once(&server, body).await;

    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("tell me something")
        .assert()
        .success();

    let user_messages = response_mock.single_request().message_input_texts("user");
    assert!(
        user_messages
            .iter()
            .any(|text| text.contains("workspace instructions")),
        "request should include workspace AGENTS.md instructions: {user_messages:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_prefers_workspace_agents_override_md() -> anyhow::Result<()> {
    let test = test_codex_exec();
    std::fs::write(test.cwd_path().join("AGENTS.md"), "base instructions")?;
    std::fs::write(
        test.cwd_path().join("AGENTS.override.md"),
        "override instructions",
    )?;

    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("resp1"),
        responses::ev_assistant_message("m1", "fixture hello"),
        responses::ev_completed("resp1"),
    ]);
    let response_mock = responses::mount_sse_once(&server, body).await;

    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("tell me something")
        .assert()
        .success();

    let user_messages = response_mock.single_request().message_input_texts("user");
    assert!(
        user_messages
            .iter()
            .any(|text| text.contains("override instructions")),
        "request should include AGENTS.override.md instructions: {user_messages:?}"
    );
    assert!(
        user_messages
            .iter()
            .all(|text| !text.contains("base instructions")),
        "request should exclude shadowed AGENTS.md instructions: {user_messages:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_fails_before_model_request_on_project_instruction_overflow() -> anyhow::Result<()> {
    let test = test_codex_exec();
    let project = test.cwd_path().canonicalize()?;
    std::fs::create_dir(project.join(".git"))?;
    std::fs::write(project.join(".git/HEAD"), "ref: refs/heads/main\n")?;
    std::fs::create_dir(project.join(".codex"))?;
    std::fs::write(
        project.join(".codex/config.toml"),
        "project_doc_max_bytes = 100\nproject_doc_overflow = 'truncate'\n\
         project_doc_fallback_filenames = []\n",
    )?;
    std::fs::write(project.join("RULES.md"), "workspace instructions")?;
    std::fs::write(
        test.home_path().join("config.toml"),
        "project_doc_max_bytes = 8\nproject_doc_overflow = 'error'\n\
         project_doc_fallback_filenames = ['RULES.md']\n",
    )?;
    set_project_trust_level(test.home_path(), &project, TrustLevel::Trusted)?;
    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("response"),
        responses::ev_assistant_message("m1", "fixture hello"),
        responses::ev_completed("response"),
    ]);
    let model = responses::mount_sse_once(&server, body).await;

    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("tell me something")
        .assert()
        .code(1)
        .stderr(contains("RULES.md"))
        .stderr(contains("8-byte `project_doc_max_bytes` budget"));

    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|request| !request.url.path().ends_with("/responses")),
        "overflow must fail before any model request"
    );

    std::fs::write(project.join("RULES.md"), "fits now")?;
    std::fs::write(test.home_path().join("AGENTS.md"), "global instructions")?;
    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("tell me something")
        .assert()
        .success();
    let user_messages = model.single_request().message_input_texts("user");
    assert!(user_messages.iter().any(|text| text.contains("fits now")));
    assert!(
        user_messages
            .iter()
            .any(|text| text.contains("global instructions"))
    );
    Ok(())
}
