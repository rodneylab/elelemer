pub static OLLAMA_DATA_JSON_CHUNKS: [&str; 33] = [
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946378,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":"The"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946379,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" user"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946380,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" says"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946380,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" \""},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946381,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":"hi"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946382,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":"\"."},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946383,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" Probably"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946384,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" wants"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946384,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" to"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946385,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" greet"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946386,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":"."},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946387,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" We"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946388,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" can"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946389,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" respond"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946390,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" with"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946392,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" a"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946393,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" friendly"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946394,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":" greeting"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946395,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"","reasoning":"."},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946402,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"Hello"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946403,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" there"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946404,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"!"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946406,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" 👋"},"finish_reason":null}]}\n\n"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946407,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" How"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946408,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" can"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946409,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" I"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946409,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" help"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946410,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" you"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946411,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":" today"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946412,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":"?"},"finish_reason":null}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946413,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":"stop"}]}

"#,
    r#"data: {"id":"chatcmpl-424","object":"chat.completion.chunk","created":1779946413,"model":"gpt-oss:20b","system_fingerprint":"fp_ollama","choices":[],"usage":{"prompt_tokens":68,"completion_tokens":41,"total_tokens":109}}

"#,
    "data: [DONE]\n\n",
];
