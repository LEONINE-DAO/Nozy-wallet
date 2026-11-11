const requestInput = document.querySelector("#request");
const form = document.querySelector("#ask-form");
const result = document.querySelector("#result");
const humanState = document.querySelector("#human-state");
const nymState = document.querySelector("#nym-state");
const walletDetail = document.querySelector("#wallet-detail");
const unlockForm = document.querySelector("#unlock-form");
const passwordInput = document.querySelector("#wallet-password");
const unlockButton = document.querySelector("#unlock-button");
const lockButton = document.querySelector("#lock-button");
const createWalletButton = document.querySelector("#create-wallet");

let pending = null;
let walletPassword = "";
let walletUnlocked = false;

refreshDesk();
refreshSetup();
refreshTee();
setInterval(refreshSetup, 8000);

unlockForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const password = passwordInput.value;
  unlockButton.disabled = true;
  const response = await fetch("/api/wallet/unlock", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ password }),
  });
  const data = await response.json();
  unlockButton.disabled = false;
  passwordInput.value = "";
  if (!data.ok) {
    walletPassword = "";
    walletUnlocked = false;
    walletDetail.textContent = data.reason || "Could not unlock the human wallet.";
    return;
  }
  walletPassword = password;
  walletUnlocked = true;
  await refreshDesk();
});

createWalletButton.addEventListener("click", async () => {
  const password = passwordInput.value;
  if (!password) return;
  createWalletButton.disabled = true;
  const response = await fetch("/api/wallet/create", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ password }),
  });
  const data = await response.json();
  createWalletButton.disabled = false;
  passwordInput.value = "";
  if (!data.ok) {
    walletPassword = "";
    walletUnlocked = false;
    walletDetail.textContent = data.reason || "Could not create the wallet.";
    return;
  }
  walletPassword = password;
  walletUnlocked = true;
  await refreshDesk();
  walletDetail.textContent = `${data.walletName || "The new wallet"} is unlocked. Open Nozy and write down its 24 words. Lowo and Wallet 1 are still on this computer.`;
});

lockButton.addEventListener("click", () => {
  walletPassword = "";
  walletUnlocked = false;
  lockButton.hidden = true;
  unlockButton.hidden = false;
  passwordInput.hidden = false;
  humanState.textContent = "Locked";
    walletDetail.textContent = "The human wallet is locked. NEAR stakes still use nozy-agent.near.";
});

async function refreshDesk() {
  const response = await fetch("/api/desk");
  const desk = await response.json();
  if (nymState) nymState.textContent = desk.nym || "Required off-box";
  if (!desk.companion) {
    humanState.textContent = "Companion offline";
    walletDetail.textContent = "Start the Nozy companion on this computer, then unlock the human wallet.";
    return;
  }
  if (!desk.exists) {
    humanState.textContent = "No wallet yet";
    walletDetail.textContent = "Create the shielded wallet in Nozy first. This desk will not invent one.";
    return;
  }
  const balance = desk.balanceZec == null ? "" : ` · ${desk.balanceZec} ZEC`;
  const role = desk.role ? ` · ${desk.role}` : "";
  const walletName = desk.walletName || "Human wallet";
  if (walletUnlocked) {
    humanState.textContent = "Unlocked";
    walletDetail.textContent = `${walletName} is unlocked${role}${balance}. It pays invoices. It does not sign NEAR.`;
    unlockButton.hidden = true;
    passwordInput.hidden = true;
    lockButton.hidden = false;
  } else if (desk.hasPassword === false) {
    humanState.textContent = "No password";
    walletDetail.textContent = `${walletName} has no password on this file. Leave the box blank and press Unlock.`;
    passwordInput.placeholder = "Leave blank";
  } else {
    humanState.textContent = "Locked";
    walletDetail.textContent = `${walletName} is on this computer${role}${balance}. Unlock it with that wallet's password.`;
    passwordInput.placeholder = "Wallet password";
  }
}

document.querySelectorAll("[data-kind]").forEach((button) => {
  button.addEventListener("click", () => {
    const body = {
      kind: button.dataset.kind,
      amount: button.dataset.amount,
      to: button.dataset.to,
      from: button.dataset.from,
      network: button.dataset.network,
      confidentiality: button.dataset.confidentiality,
      memo: "Human pays from the shielded wallet",
    };
    requestInput.value = button.textContent.trim();
    pending = body;
    submit(body, false);
  });
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const body = { text: requestInput.value };
  pending = body;
  submit(body, false);
});

async function submit(body, confirmed) {
  const response = await fetch("/api/act", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      ...body,
      confirmed,
      password: body.kind === "invoice" || /invoice|zcash|\bzec\b/i.test(body.text ?? "") ? walletPassword : undefined,
    }),
  });
  show(await response.json(), body);
}

function show(data, body) {
  result.className = "result";
  if (data.role === "earn" && data.invoice) {
    const zcash = data.invoice.asset === "ZEC";
    result.classList.add("draft");
    result.innerHTML = `
      <p class="result-kicker">Earn</p>
      <h2>${zcash ? "Shielded invoice" : "NEAR invoice"}</h2>
      <p class="quiet">${zcash
        ? "The Zcash agent can be paid on a shielded invoice. It cannot sign this, and the payment does not go through NEAR Intents."
        : "Pay nozy-agent.near in NEAR. This desk has not signed anything."}</p>
      ${facts([
        ["Amount", `${data.invoice.amount} ${data.invoice.asset}`],
        ["Purpose", data.invoice.purpose || data.purpose || ""],
        ["Pay to", data.invoice.payTo || "human Nozy wallet"],
        ["Agent may sign", "No"],
      ].filter((row) => row[1]))}
    `;
    reveal();
    return;
  }
  if (data.frost) {
    result.classList.add(data.ok ? "allowed" : "refused");
    const confirm = data.needsConfirm
      ? `<div class="confirm"><button type="button" id="confirm">Confirm the shielded spend</button><span class="quiet">A spoken yes is not enough. Two FROST signers still have to authorize it. This desk holds no spend share.</span></div>`
      : "";
    result.innerHTML = `
      <p class="result-kicker">${data.ok ? "Ready" : "Waiting"}</p>
      <h2>${escapeHtml(data.reason ?? "Shielded FROST spend")}</h2>
      <p class="quiet">${data.broadcast ? "Broadcast was requested." : "Nothing was broadcast."}</p>
      ${facts([
        ["Rail", data.rail || "zcash"],
        ["Threshold", String(data.frost.threshold ?? 2)],
        ["Agent holds a spend share", data.frost.agentHoldsShare ? "Yes" : "No"],
        ["Amount", data.amount ? `${data.amount} ZEC` : ""],
      ].filter((row) => row[1]))}
      ${confirm}
    `;
    const confirmButton = document.querySelector("#confirm");
    if (confirmButton) {
      confirmButton.addEventListener("click", () => submit(pending, true));
    }
    reveal();
    return;
  }
  if (data.invoice) {
    result.classList.add("draft");
    result.innerHTML = `
      <p class="result-kicker">Draft</p>
      <h2>Shielded invoice, waiting on the human</h2>
      <p class="quiet">${escapeHtml(data.invoice.settlement)}</p>
      <p class="quiet invoice-watch">Open. Waiting for a shielded payment.</p>
      ${facts([
        ["Amount", `${data.invoice.amount_zec} ZEC`],
        ["Payer", data.invoice.payer],
        ["Agent may sign", "No"],
        ["Payment address", data.invoice.payment_address || "Created when the human pays"],
        ["Invoice", data.invoice.invoice_id || "Draft"],
      ])}
      ${data.invoice.zcash_uri ? `<details><summary>Payment link</summary><pre>${escapeHtml(data.invoice.zcash_uri)}</pre></details>` : ""}
    `;
    reveal();
    if (data.invoice.invoice_id) watchInvoice(data.invoice.invoice_id);
    return;
  }
  if (data.ok && data.quote) {
    result.classList.add("draft");
    const deposit = data.quote.depositAddress
      ? `Deposit address ${data.quote.depositAddress}. Nothing has been sent from this computer.`
      : "Dry quote. No deposit address was created, and nothing was sent.";
    result.innerHTML = `
      <p class="result-kicker">Quote</p>
      <h2>${escapeHtml(data.from || "NEAR")} to ${escapeHtml(data.to || "USDC")}</h2>
      <p class="quiet">${escapeHtml(deposit)} The quote left on clearnet, not through Nym.</p>
      ${facts([
        ["Out", data.quote.amountOut || "Waiting on solvers"],
        ["In", data.quote.amountIn || body?.amount || ""],
        ["Payee", data.payee || ""],
        ["Purpose", data.purpose || ""],
        ["Account", data.account || "nozy-agent.near"],
        ["Confidentiality", data.confidentiality && data.confidentiality !== "public" ? data.confidentiality : ""],
      ].filter((row) => row[1]))}
      ${data.quote.dry ? `<div class="confirm"><button type="button" id="confirm">Get the deposit address</button><span class="quiet">This still does not send funds. nozy-agent.near must exist first.</span></div>` : ""}
    `;
    const confirmButton = document.querySelector("#confirm");
    if (confirmButton) {
      confirmButton.addEventListener("click", () => submit({ ...body, live: true }, true));
    }
    reveal();
    return;
  }
  if (data.ok && data.tx) {
    const action = data.tx.actions[0];
    const method = action.params.methodName ?? "Transfer";
    result.classList.add("allowed");
    result.innerHTML = `
      <p class="result-kicker">Allowed</p>
      <h2>Unsigned transaction</h2>
      <p class="quiet">The NEAR agent may sign this with its rNEAR key. This window has not signed it.</p>
      ${facts([
        ["Signer", data.tx.signerId],
        ["Receiver", data.tx.receiverId],
        ["Method", method],
        ["Attached", body?.amount ? `${body.amount} NEAR` : "see deposit"],
        ["Mainnet account", data.mainnetAccount === false ? "nozy-agent.near is not created" : ""],
      ].filter((row) => row[1]))}
      <details>
        <summary>Transaction details</summary>
        <pre>${escapeHtml(JSON.stringify(data.tx, null, 2))}</pre>
      </details>
    `;
    reveal();
    return;
  }

  result.classList.add("refused");
  const confirm = data.needsConfirm
    ? `<div class="confirm"><button type="button" id="confirm">Confirm ${escapeHtml(body?.amount ?? "")} NEAR</button><span class="quiet">A spoken yes is not enough. Press this card.</span></div>`
    : "";
  result.innerHTML = `
    <p class="result-kicker">Refused</p>
    <h2>${escapeHtml(data.reason ?? "Refused")}</h2>
    <p class="quiet">No transaction was signed.</p>
    ${confirm}
  `;
  const confirmButton = document.querySelector("#confirm");
  if (confirmButton) {
    confirmButton.addEventListener("click", () => submit(pending, true));
  }
  reveal();
}

let watchTimer = null;

function watchInvoice(id) {
  if (watchTimer) clearInterval(watchTimer);
  const tick = async () => {
    const response = await fetch(`/api/invoices/${encodeURIComponent(id)}`);
    const data = await response.json();
    const watch = result.querySelector(".invoice-watch");
    const headline = result.querySelector("h2");
    if (!watch || !headline) return;
    if (!data.ok) {
      watch.textContent = data.reason || "Waiting on the companion.";
      return;
    }
    const invoice = data.invoice;
    if (invoice.release) {
      headline.textContent = "Paid. One release.";
      watch.textContent = invoice.confirmed_txid
        ? `Confirmed ${invoice.confirmed_txid}`
        : "The companion confirmed this invoice.";
      clearInterval(watchTimer);
      watchTimer = null;
      return;
    }
    const waiting = {
      open: "Open. Waiting for a shielded payment.",
      detected: "Payment detected. Waiting for confirmation.",
      expired: "Expired. Nothing was released.",
      cancelled: "Cancelled. Nothing was released.",
    };
    watch.textContent = waiting[invoice.status] || `${invoice.status}. Nothing was released.`;
    if (invoice.status === "expired" || invoice.status === "cancelled") {
      clearInterval(watchTimer);
      watchTimer = null;
    }
  };
  tick();
  watchTimer = setInterval(tick, 5000);
}

const zcashAddress = document.querySelector("#zcash-address");
const copyZcash = document.querySelector("#copy-zcash");
const zcashFacts = document.querySelector("#zcash-facts");
const zcashDestination = document.querySelector("#zcash-destination");
const shieldButton = document.querySelector("#shield-button");
const shieldLive = document.querySelector("#shield-live");
const nearAddress = document.querySelector("#near-address");
const copyNear = document.querySelector("#copy-near");
const nearFacts = document.querySelector("#near-facts");

function shortAddress(value) {
  const text = String(value || "");
  if (text.length <= 22) return text;
  return `${text.slice(0, 12)}…${text.slice(-8)}`;
}

function setAddress(node, copyButton, full, emptyLabel) {
  const address = String(full || "");
  node.textContent = address ? shortAddress(address) : emptyLabel;
  node.title = address;
  node.dataset.full = address;
  if (copyButton) copyButton.disabled = !address;
}

async function copyAddress(button, node) {
  const full = node?.dataset.full || node?.title || node?.textContent || "";
  if (!full) return;
  try {
    await navigator.clipboard.writeText(full);
    const previous = button.textContent;
    button.textContent = "Copied";
    setTimeout(() => { button.textContent = previous; }, 1200);
  } catch {
    button.textContent = "Copy failed";
  }
}

copyZcash.addEventListener("click", () => copyAddress(copyZcash, zcashAddress));
copyNear.addEventListener("click", () => copyAddress(copyNear, nearAddress));
let shieldPoll = null;
let lastSetup = null;
let awaitingConfirm = false;

function factRows(pairs) {
  return pairs
    .filter(([, value]) => value != null && value !== "")
    .map(([name, value]) => `<div><dt>${escapeHtml(name)}</dt><dd>${escapeHtml(value)}</dd></div>`)
    .join("");
}

const teeStatus = document.querySelector("#tee-status");
const teeChecks = document.querySelector("#tee-checks");

async function refreshTee() {
  const response = await fetch("/api/tee");
  const data = await response.json();
  if (!data.ok) {
    teeStatus.textContent = data.reason || "NEAR AI Cloud did not answer.";
    teeChecks.innerHTML = "";
    return;
  }
  teeStatus.textContent = data.verified
    ? "Intel DCAP verified this gateway quote for this request."
    : "This gateway quote is not verified.";
  teeChecks.innerHTML = factRows((data.checks ?? []).map((check) => [
    check.name,
    check.ok ? (check.detail || "Passed") : (check.detail || "Not passed"),
  ]));
}

async function refreshSetup() {
  const response = await fetch("/api/setup");
  const setup = await response.json();
  lastSetup = setup;
  if (nymState && setup.nym) nymState.textContent = setup.nym;
  const zcash = setup.zcash ?? {};
  setAddress(zcashAddress, copyZcash, zcash.address, "Key file not found");
  zcashFacts.innerHTML = factRows([
    ["Balance", zcash.balanceZec ? `${zcash.balanceZec} ZEC` : ""],
    ["Zebra", zcash.up ? `Up at ${zcash.height}` : "Down"],
    ["Fee", zcash.balanceZec ? `${zcash.feeZec} ZEC` : ""],
  ]);
  const destination = setup.human?.destination;
  if (destination) {
    zcashDestination.textContent = `Notes land at ${shortAddress(destination)}`;
    zcashDestination.title = destination;
  } else {
    zcashDestination.textContent = "Unlock Nozy to show where the note lands.";
    zcashDestination.removeAttribute("title");
  }
  const ready = Boolean(zcash.attached && zcash.up && zcash.balanceZec && zcash.balanceZec !== "0.00000000" && destination);
  const phase = setup.shield?.phase;
  const busy = phase === "reading" || phase === "proving" || phase === "built" || phase === "broadcast";
  shieldButton.disabled = !ready || busy;
  shieldButton.textContent = busy ? "Shielding" : "Shield into Nozy";
  if (setup.shield?.line && !awaitingConfirm) shieldLive.textContent = setup.shield.line;
  if (busy && !shieldPoll) {
    shieldPoll = setInterval(refreshShield, 2000);
  }
  if (!busy && shieldPoll && (phase === "done" || phase === "error" || phase === "idle")) {
    clearInterval(shieldPoll);
    shieldPoll = null;
  }
  const near = setup.near ?? {};
  nearFacts.innerHTML = factRows([
    ["NEAR", near.near || "—"],
    ["rNEAR", near.rnear || "—"],
    ["RHEA", near.rheaToken || "—"],
    ["Stake", near.rhea || "lst.rhealab.near"],
  ]);
}

async function refreshShield() {
  const response = await fetch("/api/shield");
  const data = await response.json();
  if (data.shield?.line) shieldLive.textContent = data.shield.line;
  if (data.shield?.phase === "done" || data.shield?.phase === "error") {
    if (shieldPoll) clearInterval(shieldPoll);
    shieldPoll = null;
    refreshSetup();
  }
}

shieldButton.addEventListener("click", () => {
  if (awaitingConfirm) return;
  awaitingConfirm = true;
  const amount = lastSetup?.zcash?.balanceZec ?? "";
  const fee = lastSetup?.zcash?.feeZec ?? "0.00150000";
  shieldLive.innerHTML = `Shield ${escapeHtml(amount)} ZEC into the Nozy wallet. Fee ${escapeHtml(fee)} ZEC. <button type="button" id="shield-confirm">Confirm shield</button>`;
  document.querySelector("#shield-confirm").addEventListener("click", async () => {
    awaitingConfirm = false;
    shieldButton.disabled = true;
    shieldLive.textContent = "Reading the transparent coin.";
    const response = await fetch("/api/shield", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ confirmed: true }),
    });
    const data = await response.json();
    shieldLive.textContent = data.shield?.line || "Shield started.";
    if (!shieldPoll) shieldPoll = setInterval(refreshShield, 2000);
  });
});

function reveal() {
  showView("ask");
  result.scrollIntoView({ block: "nearest", behavior: "smooth" });
}

const walletPlanes = document.querySelector("#wallet-planes");
const views = ["desk", "wallets", "ask", "privacy", "shield"];

function showView(name) {
  const view = views.includes(name) ? name : "desk";
  document.querySelectorAll(".view").forEach((panel) => {
    const on = panel.dataset.view === view;
    panel.hidden = !on;
    panel.classList.toggle("is-on", on);
  });
  document.querySelectorAll(".rail button").forEach((button) => {
    button.setAttribute("aria-current", button.dataset.view === view ? "page" : "false");
  });
  const slot = document.querySelector(view === "wallets" ? "#slot-wallets" : "#slot-desk");
  if (walletPlanes && slot && (view === "desk" || view === "wallets")) {
    slot.append(walletPlanes);
    walletPlanes.classList.toggle("deep", view === "desk");
  }
  if (location.hash !== `#${view}`) history.replaceState(null, "", `#${view}`);
}

document.querySelectorAll(".rail button").forEach((button) => {
  button.addEventListener("click", () => showView(button.dataset.view));
});
window.addEventListener("hashchange", () => showView(location.hash.slice(1)));
showView(location.hash.slice(1) || "desk");

function facts(rows) {
  const items = rows
    .map(([name, value]) => `<div><dt>${escapeHtml(name)}</dt><dd>${escapeHtml(value)}</dd></div>`)
    .join("");
  return `<dl class="facts">${items}</dl>`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
