use axum::response::Html;

pub(super) async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Fleet Control</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 0; background: #f7f7f7; color: #222; }
    main { max-width: 980px; margin: auto; padding: 24px; }
    header, .agent-head, .job-top, .actions, .compute { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
    header, .job-top { justify-content: space-between; }
    .layout { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
    @media (max-width: 760px) { .layout { grid-template-columns: 1fr; } }
    .agents, .jobs { display: grid; gap: 10px; }
    .agent, .job { background: white; border: 1px solid #ddd; border-radius: 6px; padding: 12px; }
    button, input, select { font: inherit; padding: 7px; }
    button { cursor: pointer; }
    .primary { font-weight: 700; }
    .name, .job-title, .result { font-weight: 700; }
    .id, .meta, .empty, .status { color: #666; overflow-wrap: anywhere; }
    .pill { border: 1px solid #ddd; border-radius: 999px; padding: 2px 8px; }
    .completed { background: #e8f7ea; }
  </style>
</head>
<body>
  <main>
    <header>
      <h1>Fleet Control</h1>
      <button id="refresh">Refresh</button>
    </header>
    <p class="status" id="status"></p>
    <div class="layout">
      <section class="panel">
        <h2>Agents</h2>
        <div class="agents" id="agents"></div>
      </section>
      <section class="panel">
        <h2>Jobs</h2>
        <div class="jobs" id="jobs"></div>
      </section>
    </div>
  </main>
  <script>
    const agentsEl = document.querySelector("#agents");
    const jobsEl = document.querySelector("#jobs");
    const statusEl = document.querySelector("#status");

    document.querySelector("#refresh").addEventListener("click", loadAll);
    loadAll();
    setInterval(loadAll, 5000);

    async function loadAll() {
      await Promise.all([loadAgents(), loadJobs()]);
    }

    async function loadAgents() {
      const agents = await fetchJson("/fleet");
      agentsEl.innerHTML = "";
      if (agents.length === 0) {
        agentsEl.innerHTML = '<div class="empty">No agents connected.</div>';
        return;
      }
      for (const agent of agents) agentsEl.appendChild(renderAgent(agent));
    }

    async function loadJobs() {
      const jobs = await fetchJson("/jobs");
      jobsEl.innerHTML = "";
      if (jobs.length === 0) {
        jobsEl.innerHTML = '<div class="empty">No jobs yet.</div>';
        return;
      }
      for (const job of jobs) jobsEl.appendChild(renderJob(job));
    }

    function renderAgent(agent) {
      const el = document.createElement("article");
      el.className = "agent";
      el.innerHTML = `
        <div class="agent-head">
          <div>
            <div class="name">${escapeHtml(agent.name)}</div>
            <div class="id">${agent.id}</div>
          </div>
        </div>
        <div class="actions">
          <button data-kind="diagnostics">Diagnostics</button>
          <button data-kind="restart">Restart</button>
        </div>
        <div class="compute">
          <input type="number" step="any" value="12.5" aria-label="Compute number">
          <select aria-label="Calculation">
            <option value="double">Double</option>
            <option value="square">Square</option>
            <option value="square_root">Square root</option>
          </select>
          <button class="primary" data-kind="compute">Compute</button>
        </div>
      `;

      el.querySelectorAll("button[data-kind]").forEach(button => {
        button.addEventListener("click", () => queueCommand(agent.id, button.dataset.kind, el));
      });
      return el;
    }

    function renderJob(job) {
      const el = document.createElement("article");
      const completed = job.status === "completed";
      el.className = "job";
      el.innerHTML = `
        <div class="job-top">
          <div>
            <div class="job-title">${job.calculation} ${job.number}</div>
            <div class="meta">Job ${job.job_id}</div>
          </div>
          <span class="pill ${completed ? "completed" : ""}">${job.status}</span>
        </div>
        <div class="meta">Agent ${job.unit_id}</div>
        <div class="result">${completed ? `Result: ${job.result}` : "Waiting for result"}</div>
      `;
      return el;
    }

    async function queueCommand(agentId, kind, root) {
      const body = { kind };
      if (kind === "compute") {
        body.compute = {
          number: Number(root.querySelector("input").value),
          calculation: root.querySelector("select").value
        };
      }

      const command = await fetchJson(`/fleet/${agentId}/commands`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body)
      });
      statusEl.textContent = `Queued ${command.kind} for ${agentId}`;
      await loadJobs();
    }

    async function fetchJson(url, options) {
      const response = await fetch(url, options);
      if (!response.ok) throw new Error(await response.text());
      return response.json();
    }

    function escapeHtml(value) {
      return value.replace(/[&<>"']/g, char => ({
        "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
      }[char]));
    }
  </script>
</body>
</html>"##;
