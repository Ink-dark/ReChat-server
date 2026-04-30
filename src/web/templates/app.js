'use strict';

// ========== Theme ==========

const THEME_KEY = 'rechat.theme';

function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem(THEME_KEY, theme);
}

function toggleTheme() {
    const cur = document.documentElement.getAttribute('data-theme');
    const next = { light: 'dark', dark: 'auto', auto: 'light' };
    applyTheme(next[cur] || 'light');
}

(function initTheme() {
    const saved = localStorage.getItem(THEME_KEY) || 'light';
    applyTheme(saved);
})();

// ========== Hash Router ==========

const sections = ['dashboard', 'messages', 'send', 'platforms'];

function navigate(hash) {
    const name = (hash || '#dashboard').replace('#', '');
    document.querySelectorAll('.section').forEach(s => s.classList.remove('active'));
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
    const sec = document.getElementById(name);
    const nav = document.querySelector(`.nav-item[href="#${name}"]`);
    if (sec) sec.classList.add('active');
    if (nav) nav.classList.add('active');
    if (name === 'dashboard') refreshDashboard();
    if (name === 'platforms') refreshPlatforms();
}

window.addEventListener('hashchange', () => navigate(location.hash));
navigate(location.hash);

// ========== HTTP ==========

async function j(url, opts = {}) {
    try {
        const res = await fetch(url, {
            headers: { 'Content-Type': 'application/json' },
            ...opts,
        });
        const text = await res.text();
        try { return JSON.parse(text); } catch { return text; }
    } catch (e) {
        console.error('HTTP error:', url, e);
        throw e;
    }
}

// ========== WebSocket ==========

let ws = null;
let wsReconnectTimer = null;
const messageCache = [];
const MAX_CACHE = 500;

function connectWS() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${proto}//${location.host}/ws/client`;

    ws = new WebSocket(url);

    ws.onopen = () => {
        updateWSStatus(true);
        if (wsReconnectTimer) { clearTimeout(wsReconnectTimer); wsReconnectTimer = null; }
        ws.send(JSON.stringify({ type: 'subscribe', platforms: ['qq'] }));
    };

    ws.onmessage = (ev) => {
        try {
            const data = JSON.parse(ev.data);
            if (data.type === 'new_message') {
                messageCache.unshift(data.data);
                if (messageCache.length > MAX_CACHE) messageCache.length = MAX_CACHE;
                if (location.hash === '#messages' || location.hash === '' || !location.hash) {
                    renderMessages();
                }
                document.getElementById('liveDot').classList.add('active');
                setTimeout(() => document.getElementById('liveDot').classList.remove('active'), 3000);
            }
            if (data.type === 'adapter_status') {
                updateAdapterStatus(data.data.platform, data.data.content);
            }
        } catch (e) { /* ignore parse errors */ }
    };

    ws.onclose = () => {
        updateWSStatus(false);
        if (!wsReconnectTimer) {
            wsReconnectTimer = setTimeout(connectWS, 3000);
        }
    };

    ws.onerror = () => ws.close();
}

function updateWSStatus(connected) {
    const dot = document.getElementById('wsStatus');
    const footer = document.getElementById('wsFooter');
    if (dot) {
        dot.className = 'ws-status ' + (connected ? 'online' : 'offline');
        dot.title = connected ? 'WebSocket 已连接' : 'WebSocket 已断开';
    }
    if (footer) footer.textContent = 'WS: ' + (connected ? '已连接' : '已断开');
}

// ========== Dashboard ==========

async function refreshDashboard() {
    try {
        const health = await j('/api/health');
        document.getElementById('statToday').textContent = messageCache.length;
        document.getElementById('statPlatforms').textContent = '--';
        document.getElementById('statClients').textContent = '--';
        document.getElementById('statPending').textContent = '--';
        renderRecentMessages();
    } catch (e) {
        document.getElementById('statToday').textContent = '错误';
    }
}

function renderRecentMessages() {
    const tbody = document.querySelector('#recentMessages tbody');
    const recent = messageCache.slice(0, 10);
    if (!recent.length) {
        tbody.innerHTML = '<tr><td colspan="5" class="empty-row">暂无消息</td></tr>';
        return;
    }
    tbody.innerHTML = recent.map(m => {
        const time = new Date(m.created_at * 1000).toLocaleTimeString('zh-CN');
        return `<tr>
            <td><span class="badge info">${esc(m.platform)}</span></td>
            <td>${esc(m.conversation.substring(0, 20))}</td>
            <td>${esc(m.content.substring(0, 50))}</td>
            <td><span class="badge success">已接收</span></td>
            <td>${time}</td>
        </tr>`;
    }).join('');
}

// ========== Messages ==========

function renderMessages() {
    const filter = (document.getElementById('msgFilter')?.value || '').toLowerCase();
    const platform = document.getElementById('platformFilter')?.value || '';
    const tbody = document.getElementById('messagesBody');
    let list = [...messageCache];
    if (platform) list = list.filter(m => m.platform === platform);
    if (filter) list = list.filter(m => m.content.toLowerCase().includes(filter) || m.conversation.includes(filter));
    if (!list.length) {
        tbody.innerHTML = '<tr><td colspan="7" class="empty-row">暂无消息 ' + (messageCache.length ? '(已筛选)' : '(连接 WebSocket 后将实时显示)') + '</td></tr>';
        return;
    }
    tbody.innerHTML = list.map(m => {
        const time = new Date(m.created_at * 1000).toLocaleString('zh-CN');
        const sender = m.sender ? m.sender.name : '--';
        return `<tr>
            <td><span class="badge info">${esc(m.platform)}</span></td>
            <td>${esc(m.conversation.substring(0, 24))}</td>
            <td>${esc(sender)}</td>
            <td>${esc(m.content.substring(0, 40))}</td>
            <td>${esc(m.message_type)}</td>
            <td><span class="badge success">已接收</span></td>
            <td>${time}</td>
        </tr>`;
    }).join('');
}

// ========== Send ==========

async function sendMessage() {
    const platform = document.getElementById('sendPlatform').value;
    const conversation = document.getElementById('sendConversation').value.trim();
    const content = document.getElementById('sendContent').value.trim();
    const msgType = document.getElementById('sendType').value;
    const result = document.getElementById('sendResult');

    if (!conversation || !content) {
        result.innerHTML = '<span class="error">请填写会话 ID 和消息内容</span>';
        return;
    }

    try {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                type: 'send_message',
                data: { platform, conversation, content, message_type: msgType }
            }));
            result.innerHTML = '<span class="success">✓ 消息已通过 WebSocket 发送</span>';
            document.getElementById('sendContent').value = '';
        } else {
            const resp = await j('/api/messages', {
                method: 'POST',
                body: JSON.stringify({
                    message_type: msgType,
                    content,
                    recipient: conversation
                })
            });
            result.innerHTML = '<span class="success">✓ 消息已创建: ' + esc(resp.id) + '</span>';
        }
    } catch (e) {
        result.innerHTML = '<span class="error">✗ 发送失败: ' + esc(e.message) + '</span>';
    }
}

// ========== Platforms ==========

const adapterStates = {};

function updateAdapterStatus(platform, status) {
    adapterStates[platform] = status;
    if (location.hash === '#platforms') renderAdapterCards();
}

function renderAdapterCards() {
    const grid = document.getElementById('adapterGrid');
    const entries = Object.entries(adapterStates);
    if (!entries.length) {
        grid.innerHTML = '<div class="card stat-card muted"><div class="stat-label">无已注册平台</div><div class="stat-value">--</div></div>';
        return;
    }
    grid.innerHTML = entries.map(([name, status]) => {
        const cls = status === 'Connected' ? 'success' : 'warning';
        return `<div class="card stat-card">
            <div class="stat-label">${esc(name)}</div>
            <div class="stat-value" style="font-size:18px"><span class="badge ${cls}">${esc(status)}</span></div>
        </div>`;
    }).join('');
}

async function refreshPlatforms() {
    renderAdapterCards();
    try {
        const health = await j('/api/health');
        if (health && health.status !== 'ok') {
            document.getElementById('adapterGrid').innerHTML = '<div class="card stat-card muted"><div class="stat-label">服务异常</div></div>';
        }
    } catch (e) { /* ignore */ }
}

// ========== Utility ==========

function esc(s) {
    if (!s) return '';
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// ========== Boot ==========

connectWS();
refreshDashboard();

// Periodically refresh dashboard stats
setInterval(() => {
    const hash = location.hash || '#dashboard';
    if (hash === '#dashboard') refreshDashboard();
}, 5000);
