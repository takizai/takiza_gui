use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteDecision {
    pub category: String,
    pub complexity: String,
    pub benchmark_rationale: String,
    pub selected_model: String,
    pub estimated_tokens: Option<usize>,
    #[serde(default)]
    pub source: Option<String>,
}

/// Query the local or remote Takiza MoA LoRA router daemon.
/// Supports local daemon (http://127.0.0.1:8088/route) or Colab cloud tunnel (TAKIZA_MOA_ROUTER_URL).
/// If the daemon is unreachable, smoothly falls back to heuristic benchmark routing.
pub async fn resolve_moa_route(prompt: &str, endpoint: Option<&str>) -> RouteDecision {
    let env_url = std::env::var("TAKIZA_MOA_ROUTER_URL").ok().filter(|s| !s.trim().is_empty());
    let config_url = crate::core::config::Config::config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|v| v.get("moa_router_url").and_then(|u| u.as_str().map(|s| s.trim().to_string())))
        .filter(|s| !s.is_empty());

    let url = endpoint
        .or(env_url.as_deref())
        .or(config_url.as_deref())
        .unwrap_or("http://127.0.0.1:8088/route");
    if let Some(mut decision) = query_daemon(prompt, url).await {
        if decision.source.is_none() {
            decision.source = Some(if url.contains("trycloudflare") || url.starts_with("https://") {
                "Colab GPU".to_string()
            } else {
                "Local Daemon".to_string()
            });
        }

        // Safety clamp: never spend expensive flagship models on greetings or trivial queries
        let comp = decision.complexity.to_lowercase();
        let cat = decision.category.to_lowercase();
        let trimmed_len = prompt.trim().chars().count();
        if comp == "low" || comp == "easy" || cat == "greeting" || cat == "chat" || cat == "general" || trimmed_len < 15 {
            if decision.selected_model.contains("astra") || decision.selected_model.contains("sol") || decision.selected_model.contains("opus") {
                decision.selected_model = "ag/gemini-3.7-flash-high".to_string();
                decision.complexity = "low".to_string();
            }
        }

        return decision;
    }

    // Heuristic fallback matching our trained benchmark taxonomy
    let mut fb = fallback_route(prompt);
    fb.source = Some("Offline Fallback".to_string());
    fb
}

async fn query_daemon(prompt: &str, url: &str) -> Option<RouteDecision> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(25000))
        .build()
        .ok()?;

    let body = serde_json::json!({ "prompt": prompt });
    let res = client.post(url).json(&body).send().await.ok()?;
    if res.status().is_success() {
        res.json::<RouteDecision>().await.ok()
    } else {
        None
    }
}

/// Fallback heuristic that matches the exact dataset and benchmark taxonomy of our LoRA model
pub fn fallback_route(prompt: &str) -> RouteDecision {
    let lower = prompt.to_lowercase();

    // 1. Ultra hard: distributed systems, consensus, atomic allocators, cryptography, SIMD, AVX
    if lower.contains("raft") || lower.contains("consensus") || lower.contains("консенсус")
        || lower.contains("lock-free") || lower.contains("allocator") || lower.contains("zero-knowledge")
        || lower.contains("snark") || lower.contains("распределенн") || lower.contains("2pc")
        || lower.contains("simd") || lower.contains("avx") || lower.contains("интринсик")
        || lower.contains("spmm") || lower.contains("kademlia") || lower.contains("dht") {
        return RouteDecision {
            category: "system_architecture".to_string(),
            complexity: "ultra_hard".to_string(),
            benchmark_rationale: "Frontier-level distributed architecture, SIMD or formal logic (AIME/GPQA profile)".to_string(),
            selected_model: "cx/gpt-6-astra".to_string(),
            estimated_tokens: Some(3000),
            source: None,
        };
    }

    // 2. Security Audit, Reverse Engineering & Vulnerability Assessment -> xAI Grok 4.7
    if lower.contains("grok") || lower.contains("уязвимост") || lower.contains("security")
        || lower.contains("безопасн") || lower.contains("exploit") || lower.contains("эксплоит")
        || lower.contains("реверс") || lower.contains("reverse engineering") || lower.contains("декомпил")
        || lower.contains("penetration") || lower.contains("cve") || lower.contains("injection")
        || lower.contains("инъекци") || lower.contains("xss") || lower.contains("csrf") {
        return RouteDecision {
            category: "security_auditing".to_string(),
            complexity: "hard".to_string(),
            benchmark_rationale: "Deep security audit, reverse engineering & vulnerability assessment (xAI Grok 4.7 profile)".to_string(),
            selected_model: "xai/grok-4.7".to_string(),
            estimated_tokens: Some(2500),
            source: None,
        };
    }

    // 3. Hard: concurrency, deadlocks, borrow checker, multi-file refactoring, async state machines, FFI
    if lower.contains("deadlock") || lower.contains("race condition") || lower.contains("гонк")
        || lower.contains("mpsc") || lower.contains("borrow") || lower.contains("use-after-free")
        || lower.contains("многофайл") || lower.contains("cancellationtoken") || lower.contains("ffi") {
        return RouteDecision {
            category: "concurrency_debugging".to_string(),
            complexity: "hard".to_string(),
            benchmark_rationale: "Complex async state machine, FFI safety & memory ownership (SWE-bench Verified profile)".to_string(),
            selected_model: "cc/claude-opus-5".to_string(),
            estimated_tokens: Some(2500),
            source: None,
        };
    }

    // 3. Agentic SWE & Multimodal: dependency conflicts, Cargo.lock, UI, CSS, верстка, frontend -> Sonnet 5
    if lower.contains("конфликт") || lower.contains("зависимост") || lower.contains("cargo.lock")
        || lower.contains("верстк") || lower.contains("ui") || lower.contains("css")
        || lower.contains("flexbox") || lower.contains("grid") || lower.contains("скриншот")
        || lower.contains("компонент") || lower.contains("frontend") {
        return RouteDecision {
            category: "dependency_resolution".to_string(),
            complexity: "hard".to_string(),
            benchmark_rationale: "Subtle dependency tree conflict resolution & multimodal UI engineering (SWE-bench profile)".to_string(),
            selected_model: "cc/claude-sonnet-5".to_string(),
            estimated_tokens: Some(1800),
            source: None,
        };
    }

    // 4. Medium: unit tests, CRUD, backend boilerplate, standard functions
    if lower.contains("тест") || lower.contains("test") || lower.contains("crud")
        || lower.contains("endpoint") || lower.contains("маршрут") || lower.contains("handler")
        || lower.contains("валидац") || lower.contains("парсинг") || lower.contains("parse") {
        return RouteDecision {
            category: "routine_backend".to_string(),
            complexity: "medium".to_string(),
            benchmark_rationale: "Standard enterprise backend/test generation, high speed needed (HumanEval 95%+ profile)".to_string(),
            selected_model: "ag/gemini-3.7-flash-high".to_string(),
            estimated_tokens: Some(1200),
            source: None,
        };
    }

    // 5. Default fast coding & inspection
    RouteDecision {
        category: "general_code_edit".to_string(),
        complexity: "medium".to_string(),
        benchmark_rationale: "General coding, search & localized refactoring (balanced quality/latency)".to_string(),
        selected_model: "ag/gemini-3.7-flash-high".to_string(),
        estimated_tokens: Some(1000),
        source: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_moa_remote_or_fallback_resolution() {
        let decision = resolve_moa_route("У нас deadlock в Tokio mpsc канале", None).await;
        assert!(!decision.selected_model.is_empty());
        assert!(decision.source.is_some());
        println!("Resolved via: {:?} -> {}", decision.source, decision.selected_model);
    }

    #[test]
    fn test_fallback_route_grok_security() {
        let decision = fallback_route("Проведи аудит безопасности и поиск уязвимостей SQL injection и XSS в коде");
        assert_eq!(decision.selected_model, "xai/grok-4.7");
        assert_eq!(decision.category, "security_auditing");
    }
}

