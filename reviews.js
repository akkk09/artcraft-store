(() => {
  const REPO = "akkk09/artcraft-store";
  const API = "https://api.github.com/repos/" + REPO + "/issues?state=all&per_page=100";
  const ISSUE_NEW = "https://github.com/" + REPO + "/issues/new?template=plugin-review.yml&title=";
  const style = document.createElement("style");
  style.textContent = `
    .review-panel{display:flex;align-items:center;justify-content:space-between;gap:12px;margin:14px 0 12px;padding:11px 0;border-top:1px solid #ffffff12;border-bottom:1px solid #ffffff12}
    .review-score{font-size:13px;color:var(--text);font-weight:700}.review-score .stars{color:var(--lime);letter-spacing:.05em}.review-count{font-size:12px;color:var(--muted);font-weight:400}
    .review-actions{display:flex;gap:8px;align-items:center;flex-wrap:wrap}.review-btn{border:1px solid var(--line);border-radius:7px;background:transparent;color:var(--text);padding:7px 9px;font-size:12px;font-weight:650;cursor:pointer;text-decoration:none}.review-btn:hover{border-color:var(--lime);color:var(--lime)}
    .review-list{display:none;margin:0 0 14px;padding:0;list-style:none}.review-list.open{display:block}.review-item{padding:10px 0;border-bottom:1px solid #ffffff12}.review-item:last-child{border-bottom:0}.review-item strong{font-size:12px}.review-item p{min-height:0!important;margin:4px 0 0!important;font-size:12px!important;white-space:pre-wrap}.review-meta{font-size:11px;color:var(--muted);margin-top:3px}
    .review-status{font-size:12px;color:var(--muted);padding:8px 0}.review-disclaimer{font-size:11px;color:var(--muted);margin:0 0 12px}
  `;
  document.head.appendChild(style);
  const esc = value => String(value ?? "").replace(/[&<>"']/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
  const field = (body, label) => {
    const lines = String(body || "").split(/\\r?\\n/);
    const heading = "### " + label.trim().toLowerCase();
    const start = lines.findIndex(line => line.trim().toLowerCase() === heading);
    if (start < 0) return "";
    const value = [];
    for (let i = start + 1; i < lines.length; i++) {
      if (/^###\\s/.test(lines[i])) break;
      value.push(lines[i]);
    }
    return value.join("\\n").trim();
  };
  const ratingOf = issue => {
    const raw = field(issue.body, "Overall rating");
    const match = raw.match(/[1-5]/);
    return match ? Number(match[0]) : 0;
  };
  const nameOf = issue => field(issue.body, "Plugin or script name").toLowerCase();
  const idOf = issue => field(issue.body, "Catalog item ID (if shown)");
  const reviewText = issue => field(issue.body, "Your review") || "No written review provided.";
  let reviews = [];
  let reviewsReady = false;
  function enhanceCards() {
    document.querySelectorAll("#plugin-grid .card").forEach(card => {
      if (card.dataset.reviewsReady) return;
      const name = card.querySelector("h3")?.textContent?.trim();
      if (!name) return;
      card.dataset.reviewsReady = "true";
      const panel = document.createElement("div");
      panel.className = "review-panel";
      panel.innerHTML = '<div class="review-score">Loading reviews…</div><div class="review-actions"></div>';
      const list = document.createElement("ul");
      list.className = "review-list";
      list.setAttribute("aria-label", "Community reviews for " + name);
      const foot = card.querySelector(".card-foot");
      if (foot) card.insertBefore(panel, foot); else card.append(panel);
      card.insertBefore(list, panel.nextSibling);
      const actions = panel.querySelector(".review-actions");
      const reviewUrl = ISSUE_NEW + encodeURIComponent("Review: " + name);
      const add = document.createElement("a");
      add.className = "review-btn";
      add.href = reviewUrl;
      add.target = "_blank";
      add.rel = "noopener noreferrer";
      add.textContent = "Write a review ↗";
      add.setAttribute("aria-label", "Write a review for " + name + " on GitHub");
      actions.append(add);
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.className = "review-btn";
      toggle.textContent = "Read reviews";
      toggle.addEventListener("click", () => {
        const open = list.classList.toggle("open");
        toggle.textContent = open ? "Hide reviews" : "Read reviews";
      });
      actions.append(toggle);
      card.dataset.reviewName = name;
    });
    renderReviews();
  }
  function renderReviews() {
    document.querySelectorAll("#plugin-grid .card[data-review-name]").forEach(card => {
      const name = card.dataset.reviewName;
      const matches = reviews.filter(issue => {
        const n = nameOf(issue);
        const id = idOf(issue).toLowerCase();
        return n === name.toLowerCase() || (card.dataset.pluginId && id === card.dataset.pluginId.toLowerCase());
      });
      const rated = matches.map(issue => ({issue, rating: ratingOf(issue)})).filter(x => x.rating > 0);
      const average = rated.length ? rated.reduce((sum, x) => sum + x.rating, 0) / rated.length : 0;
      const score = card.querySelector(".review-score");
      if (score) score.innerHTML = rated.length
        ? '<span class="stars" aria-label="' + average.toFixed(1) + ' out of 5 stars">' + "★".repeat(Math.round(average)) + "☆".repeat(5 - Math.round(average)) + '</span> ' + average.toFixed(1) + ' <span class="review-count">(' + rated.length + ' rating' + (rated.length === 1 ? '' : 's') + ')</span>'
        : '<span class="review-count">' + (reviewsReady ? "No reviews yet" : "Reviews unavailable") + '</span>';
      const list = card.querySelector(".review-list");
      if (!list) return;
      if (!reviewsReady) {
        list.innerHTML = '<li class="review-status">Could not load public reviews. You can still submit one through GitHub.</li>';
        return;
      }
      list.innerHTML = matches.length ? matches.slice(0, 5).map(issue => {
        const rating = ratingOf(issue);
        const version = field(issue.body, "Item version tested");
        const reliability = field(issue.body, "Did it work as expected?");
        const date = issue.created_at ? new Date(issue.created_at).toLocaleDateString() : "";
        return '<li class="review-item"><strong>' + esc(issue.title.replace(/^\[Plugin Review\]\s*/i, "")) + '</strong><div class="review-meta">' + (rating ? esc("★".repeat(rating) + "☆".repeat(5-rating) + " · ") : "") + esc(version ? "v" + version + " · " : "") + esc(reliability ? reliability + " · " : "") + esc(date) + '</div><p>' + esc(reviewText(issue)) + '</p><div class="review-meta"><a href="' + esc(issue.html_url) + '" target="_blank" rel="noopener noreferrer">View on GitHub ↗</a></div></li>';
      }).join("") : '<li class="review-status">No community reviews yet. Be the first to share your experience.</li>';
    });
  }
  const observer = new MutationObserver(enhanceCards);
  const grid = document.querySelector("#plugin-grid");
  if (grid) observer.observe(grid, {childList:true, subtree:true});
  enhanceCards();
  fetch(API, {headers:{"Accept":"application/vnd.github+json"}})
    .then(response => { if (!response.ok) throw new Error("GitHub API returned " + response.status); return response.json(); })
    .then(items => {
      reviews = items.filter(issue => !issue.pull_request && /^\[Plugin Review\]/i.test(issue.title));
      reviewsReady = true;
      renderReviews();
    })
    .catch(() => { reviewsReady = false; renderReviews(); });
})();