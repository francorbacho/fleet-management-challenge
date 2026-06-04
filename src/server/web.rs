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
    body { font-family: system-ui, sans-serif; margin: 24px; color: #222; }
    header { display: flex; gap: 12px; align-items: center; }
    h1 { margin: 0; font-size: 24px; }
    h2 { margin: 24px 0 8px; font-size: 18px; }
    .layout { display: grid; gap: 24px; grid-template-columns: 1fr 1fr; align-items: start; }
    table { width: 100%; border-collapse: collapse; margin-bottom: 16px; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; vertical-align: middle; }
    th { background: #f3f3f3; }
    tr.accepted { background: #fff7cc; }
    tr.succeed { background: #e8f7ea; }
    tr.failed { background: #fdecec; }
    button, input, select { font: inherit; padding: 6px; }
    button { cursor: pointer; }
    .status { min-height: 24px; color: #555; }
    .id { font-family: ui-monospace, SFMono-Regular, monospace; }
    tr.disconnected { opacity: 0.5; }
    .status-badge { padding: 2px 8px; border-radius: 4px; font-size: 12px; }
    .status-connected { background: #e8f7ea; color: #2d6a30; }
    .status-disconnected { background: #fdecec; color: #a33; }
    @media (max-width: 900px) { .layout { grid-template-columns: 1fr; } }
  </style>
</head>
<body>
  <header>
    <h1>Fleet Control</h1>
    <button id="refresh">Refresh</button>
  </header>
  <p class="status" id="status"></p>

  <div class="layout">
    <section>
      <h2>Agents</h2>
      <table>
        <thead>
          <tr>
            <th>Agent id</th>
            <th>Name</th>
            <th>Status</th>
            <th>Command</th>
            <th>Double</th>
          </tr>
        </thead>
        <tbody id="agents"></tbody>
      </table>
    </section>

    <section>
      <h2>Jobs</h2>
      <table>
        <thead>
          <tr>
            <th>Job id</th>
            <th>Agent id</th>
            <th>Command</th>
            <th>State</th>
            <th>Result</th>
          </tr>
        </thead>
        <tbody id="jobs"></tbody>
      </table>
    </section>
  </div>

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
      if (agents.length === 0) {
        agentsEl.replaceChildren(emptyRow(5, "No agents connected."));
        return;
      }

      agentsEl.querySelectorAll("tr:not([data-agent-id])").forEach(row => row.remove());
      const seen = new Set();
      for (const agent of agents) {
        const id = String(agent.id);
        seen.add(id);
        const row = agentsEl.querySelector(`tr[data-agent-id="${id}"]`);
        if (row) {
          row.querySelector("[data-agent-name]").textContent = agent.name;
          updateAgentStatus(row, agent.status);
        } else {
          agentsEl.appendChild(renderAgent(agent));
        }
      }
      agentsEl.querySelectorAll("tr[data-agent-id]").forEach(row => {
        if (!seen.has(row.dataset.agentId)) row.remove();
      });
    }

    async function loadJobs() {
      const jobs = await fetchJson("/jobs");
      jobsEl.replaceChildren();
      if (jobs.length === 0) {
        jobsEl.appendChild(emptyRow(5, "No jobs yet."));
        return;
      }
      for (const job of jobs) jobsEl.appendChild(renderJob(job));
    }

    function renderAgent(agent) {
      const row = document.createElement("tr");
      row.dataset.agentId = String(agent.id);
      row.innerHTML = `
        <td class="id">${formatAgentId(agent.id)}</td>
        <td data-agent-name></td>
        <td data-agent-status></td>
        <td>
          <button data-kind="diagnostics">Diagnostics</button>
          <button data-kind="restart">Restart</button>
          <button data-kind="exit">Exit</button>
        </td>
        <td>
          <input type="number" step="any" value="12.5" aria-label="Double number">
          <button data-kind="double">Double</button>
        </td>
      `;
      row.querySelector("[data-agent-name]").textContent = agent.name;
      updateAgentStatus(row, agent.status);
      row.querySelectorAll("button[data-kind]").forEach(button => {
        button.addEventListener("click", () => queueCommand(agent.id, button.dataset.kind, row));
      });
      return row;
    }

    function emptyRow(colspan, message) {
      const row = document.createElement("tr");
      row.innerHTML = `<td colspan="${colspan}">${message}</td>`;
      return row;
    }

    function renderJob(job) {
      const row = document.createElement("tr");
      row.className = job.status;
      row.innerHTML = `
        <td class="id">${formatJobId(job.job_id)}</td>
        <td class="id">${formatAgentId(job.agent_id)}</td>
        <td>${formatCommand(job.command)}</td>
        <td>${job.status}</td>
        <td>${renderJobResult(job)}</td>
      `;
      return row;
    }

    function formatCommand(command) {
      if (typeof command === "string") return command;
      if (command.double !== undefined) return `double(${command.double})`;
      return JSON.stringify(command);
    }

    function renderJobResult(job) {
      if (job.status === "succeed") return job.result;
      if (job.status === "accepted") return "accepted";
      if (job.status === "failed") return "failed";
      return "";
    }

    async function queueCommand(agentId, kind, row) {
      let request;
      if (kind === "double") {
        request = { double: Number(row.querySelector("input").value) };
      } else {
        request = kind;
      }

      const command = await fetchJson(`/fleet/${agentId}/commands`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(request)
      });
      statusEl.textContent = `Queued ${formatCommand(command.request)} for ${formatAgentId(agentId)}`;
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

    function formatAgentId(id) {
      return `a#${formatId(id)}`;
    }

    function formatJobId(id) {
      return `j#${formatId(id)}`;
    }

    function formatId(id) {
      return String(id);
    }

    function updateAgentStatus(row, status) {
      const cell = row.querySelector("[data-agent-status]");
      const isConnected = status === "connected";
      cell.innerHTML = `<span class="status-badge ${isConnected ? 'status-connected' : 'status-disconnected'}">${status}</span>`;
      row.classList.toggle("disconnected", !isConnected);
    }
  </script>
</body>
</html>"##;
