use anyhow::Result;
use testcontainers::{clients::Cli, core::WaitFor, GenericImage};

use code_context_graph_graph::GraphClient;

#[test]
#[ignore]
fn falkordb_with_testcontainers_persist_and_query() -> Result<()> {
    // Permite habilitar el test explícitamente en CI/locally
    if std::env::var("ENABLE_DOCKER_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping: ENABLE_DOCKER_TESTS != 1");
        return Ok(());
    }

    // Comprueba si Docker está disponible; si no, salta el test de forma limpia
    match std::process::Command::new("docker").arg("ps").status() {
        Ok(status) if status.success() => {}
        _ => {
            eprintln!("skipping: Docker daemon not available for testcontainers");
            return Ok(());
        }
    }

    // Requiere Docker daemon activo
    let docker = Cli::default();

    // Imagen oficial de FalkorDB, expone 6379 y espera readiness
    let image = GenericImage::new("falkordb/falkordb", "latest")
        .with_exposed_port(6379)
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"));

    let node = docker.run(image);

    // Puerto publicado por Docker (host será localhost en Linux)
    let port = node.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}", port);

    let graph_name = "ccg_it_test";
    let client = GraphClient::new_with_redis(&url, graph_name)?;

    // Limpia el grafo si existiera de ejecuciones previas
    let _ = client.execute("CALL db.dropGraph('ccg_it_test', false)")?;

    // Inserta nodos y relación
    client.persist_queries(&[
        "MERGE (f:File {path: 'src/main.rs'})".to_string(),
        "MERGE (fn:Function {name: 'foo', signature: 'foo()'})".to_string(),
        "MERGE (f)-[:CONTAINS]->(fn)".to_string(),
    ])?;

    // Consulta de verificación
    let res = client.execute(
        "MATCH (f:File {path:'src/main.rs'})-[:CONTAINS]->(fn:Function {name:'foo'}) RETURN count(*)",
    )?;

    match res {
        redis::Value::Bulk(ref arr) => {
            assert!(
                !arr.is_empty(),
                "query returned empty result set: {:?}",
                arr
            );
        }
        other => panic!("unexpected redis value: {:?}", other),
    }

    Ok(())
}
