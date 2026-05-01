/*
  设计参考: Tomato-Novel-Downloader (MIT)
  https://gitcode.com/gh_mirrors/to/Tomato-Novel-Downloader
  借鉴: Hash 路由、localStorage 主题持久化、事件委托、innerHTML 模板渲染、WebSocket 自动重连
*/

'use strict';

// ========== Token Auth ==========

var accessToken = (function () {
    var stored = localStorage.getItem('rechat.token');
    if (stored) return stored;
    var urlToken = new URLSearchParams(location.search).get('token');
    if (urlToken) {
        localStorage.setItem('rechat.token', urlToken);
        history.replaceState(null, '', location.pathname + location.hash);
        return urlToken;
    }
    return null;
})();

function doLogin() {
    var input = document.getElementById('tokenInput');
    var errEl = document.getElementById('loginError');
    var token = input.value.trim();
    if (!token) {
        errEl.textContent = '请输入 Token';
        return;
    }
    j('/api/auth/verify', {
        method: 'POST',
        body: JSON.stringify({ token: token })
    }).then(function (resp) {
        if (resp.valid) {
            localStorage.setItem('rechat.token', token);
            location.reload();
        } else {
            errEl.textContent = 'Token 无效，请重试';
        }
    }).catch(function () {
        errEl.textContent = '验证失败，请检查服务端是否运行';
    });
}

function hideLogin() {
    var overlay = document.getElementById('loginOverlay');
    if (overlay) overlay.style.display = 'none';
}

// ========== Theme ==========

var THEME_KEY = 'rechat.theme';

function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem(THEME_KEY, theme);
}

function toggleTheme() {
    var cur = document.documentElement.getAttribute('data-theme');
    var next = { light: 'dark', dark: 'auto', auto: 'light' };
    applyTheme(next[cur] || 'light');
}

(function initTheme() {
    applyTheme(localStorage.getItem(THEME_KEY) || 'light');
})();

// ========== Hash Router ==========

function navigate(hash) {
    var name = (hash || '#dashboard').replace('#', '');
    document.querySelectorAll('.section').forEach(function (s) { s.classList.remove('active'); });
    document.querySelectorAll('.nav-item').forEach(function (n) { n.classList.remove('active'); });
    var sec = document.getElementById(name);
    var nav = document.querySelector('.nav-item[href="#' + name + '"]');
    if (sec) sec.classList.add('active');
    if (nav) nav.classList.add('active');
    if (name === 'dashboard') refreshDashboard();
    if (name === 'platforms') refreshPlatforms();
}

window.addEventListener('hashchange', function () { navigate(location.hash); });

// ========== HTTP ==========

function tokenParam() {
    return accessToken ? '&access_token=' + encodeURIComponent(accessToken) : '';
}

function j(url, opts) {
    opts = opts || {};
    opts.headers = opts.headers || {};
    opts.headers['Content-Type'] = 'application/json';
    if (url.indexOf('?') === -1 && tokenParam()) {
        url = url + '?' + tokenParam().substring(1);
    }
    return fetch(url, opts).then(function (res) {
        return res.text().then(function (text) {
            try { return JSON.parse(text); } catch (e) { return text; }
        });
    });
}

// ========== WebSocket ==========

var ws = null;
var wsReconnectTimer = null;
var messageCache = [];
var MAX_CACHE = 500;

function connectWS() {
    if (!accessToken) return;
    var proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    var url = proto + '//' + location.host + '/ws/client?access_token=' + encodeURIComponent(accessToken);

    ws = new WebSocket(url);

    ws.onopen = function () {
        updateWSStatus(true);
        if (wsReconnectTimer) { clearTimeout(wsReconnectTimer); wsReconnectTimer = null; }
        ws.send(JSON.stringify({ type: 'subscribe', platforms: ['qq'] }));
    };

    ws.onmessage = function (ev) {
        try {
            var data = JSON.parse(ev.data);
            if (data.type === 'new_message') {
                messageCache.unshift(data.data);
                if (messageCache.length > MAX_CACHE) messageCache.length = MAX_CACHE;
                if (location.hash === '#messages' || location.hash === '' || !location.hash) {
                    renderMessages();
                }
                document.getElementById('liveDot').classList.add('active');
                setTimeout(function () { document.getElementById('liveDot').classList.remove('active'); }, 3000);
            }
            if (data.type === 'adapter_status') {
                updateAdapterStatus(data.data.platform, data.data.content);
            }
        } catch (e) { /* ignore */ }
    };

    ws.onclose = function () {
        updateWSStatus(false);
        if (!wsReconnectTimer) {
            wsReconnectTimer = setTimeout(connectWS, 3000);
        }
    };

    ws.onerror = function () { ws.close(); };
}

function updateWSStatus(connected) {
    var dot = document.getElementById('wsStatus');
    var footer = document.getElementById('wsFooter');
    if (dot) {
        dot.className = 'ws-status ' + (connected ? 'online' : 'offline');
        dot.title = connected ? 'WebSocket 已连接' : 'WebSocket 已断开';
    }
    if (footer) footer.textContent = 'WS: ' + (connected ? '已连接' : '已断开');
}

// ========== Dashboard ==========

function refreshDashboard() {
    j('/api/stats').then(function (stats) {
        document.getElementById('statPending').textContent = stats.pending || 0;
        document.getElementById('statSending').textContent = stats.sending || 0;
        document.getElementById('statSent').textContent = stats.sent || 0;
        document.getElementById('statFailed').textContent = stats.failed || 0;
        document.getElementById('statCanceled').textContent = stats.canceled || 0;
        document.getElementById('statTotal').textContent = stats.total || 0;
    }).catch(function () {
        document.getElementById('statPending').textContent = 'ERR';
    });
    renderRecentMessages();
}

function renderRecentMessages() {
    var tbody = document.querySelector('#recentMessages tbody');
    var recent = messageCache.slice(0, 10);
    if (!recent.length) {
        tbody.innerHTML = '<tr><td colspan="5" class="empty-row">暂无消息</td></tr>';
        return;
    }
    tbody.innerHTML = recent.map(function (m) {
        var time = new Date(m.created_at * 1000).toLocaleTimeString('zh-CN');
        return '<tr>' +
            '<td><span class="badge info">' + esc(m.platform) + '</span></td>' +
            '<td>' + esc((m.conversation || '').substring(0, 20)) + '</td>' +
            '<td>' + esc((m.content || '').substring(0, 50)) + '</td>' +
            '<td>' + statusBadge(m.status || 'Pending') + '</td>' +
            '<td>' + time + '</td>' +
            '</tr>';
    }).join('');
}

// ========== Messages ==========

function renderMessages() {
    var filter = (document.getElementById('msgFilter') ? document.getElementById('msgFilter').value : '').toLowerCase();
    var platform = document.getElementById('platformFilter') ? document.getElementById('platformFilter').value : '';
    var tbody = document.getElementById('messagesBody');
    var list = messageCache.slice();
    if (platform) list = list.filter(function (m) { return m.platform === platform; });
    if (filter) list = list.filter(function (m) { return m.content.toLowerCase().includes(filter) || m.conversation.includes(filter); });
    if (!list.length) {
        tbody.innerHTML = '<tr><td colspan="7" class="empty-row">暂无消息 ' + (messageCache.length ? '(已筛选)' : '(连接 WebSocket 后将实时显示)') + '</td></tr>';
        return;
    }
    tbody.innerHTML = list.map(function (m) {
        var time = new Date(m.created_at * 1000).toLocaleString('zh-CN');
        var sender = m.sender ? m.sender.name : '--';
        return '<tr>' +
            '<td><span class="badge info">' + esc(m.platform) + '</span></td>' +
            '<td>' + esc((m.conversation || '').substring(0, 24)) + '</td>' +
            '<td>' + esc(sender) + '</td>' +
            '<td>' + esc((m.content || '').substring(0, 40)) + '</td>' +
            '<td>' + esc(m.message_type) + '</td>' +
            '<td>' + statusBadge(m.status || 'Pending') + '</td>' +
            '<td>' + time + '</td>' +
            '</tr>';
    }).join('');
}

// ========== Send ==========

function sendMessage() {
    var platform = document.getElementById('sendPlatform').value;
    var conversation = document.getElementById('sendConversation').value.trim();
    var content = document.getElementById('sendContent').value.trim();
    var msgType = document.getElementById('sendType').value;
    var result = document.getElementById('sendResult');

    if (!conversation || !content) {
        result.innerHTML = '<span class="error">请填写会话 ID 和消息内容</span>';
        return;
    }

    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: 'send_message',
            data: { platform: platform, conversation: conversation, content: content, message_type: msgType }
        }));
        result.innerHTML = '<span class="success">✓ 消息已发送</span>';
        document.getElementById('sendContent').value = '';
    } else {
        j('/api/messages', {
            method: 'POST',
            body: JSON.stringify({
                message_type: msgType,
                content: content,
                recipient: conversation
            })
        }).then(function (resp) {
            result.innerHTML = '<span class="success">✓ 消息已创建: ' + esc(resp.id) + '</span>';
        }).catch(function (e) {
            result.innerHTML = '<span class="error">✗ 发送失败: ' + esc(e.message) + '</span>';
        });
    }
}

// ========== Platforms ==========

var adapterStates = {};

function updateAdapterStatus(platform, status) {
    adapterStates[platform] = status;
    if (location.hash === '#platforms') renderAdapterCards();
}

function renderAdapterCards() {
    var grid = document.getElementById('adapterGrid');
    var entries = Object.entries(adapterStates);
    if (!entries.length) {
        grid.innerHTML = '<div class="card stat-card muted"><div class="stat-label">无已注册平台</div><div class="stat-value">--</div></div>';
        return;
    }
    grid.innerHTML = entries.map(function (entry) {
        var name = entry[0];
        var status = entry[1];
        var cls = status === 'Connected' ? 'success' : 'warning';
        return '<div class="card stat-card">' +
            '<div class="stat-label">' + esc(name) + '</div>' +
            '<div class="stat-value" style="font-size:18px"><span class="badge ' + cls + '">' + esc(status) + '</span></div>' +
            '</div>';
    }).join('');
}

function refreshPlatforms() {
    renderAdapterCards();
}

// ========== Utility ==========

function esc(s) {
    if (!s) return '';
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function statusBadge(status) {
    var map = {
        Sent: 'success',
        Pending: 'warning',
        Sending: 'info',
        Failed: 'danger',
        Canceled: 'danger'
    };
    var label = {
        Sent: '已发送',
        Pending: '待发送',
        Sending: '发送中',
        Failed: '失败',
        Canceled: '已取消'
    };
    var cls = map[status] || 'info';
    return '<span class="badge ' + cls + '">' + (label[status] || status || '未知') + '</span>';
}

// ========== Boot ==========

if (accessToken) {
    hideLogin();
    navigate(location.hash || '#dashboard');
    connectWS();
    refreshDashboard();

    // Auto-check URL token param to validate
    var urlToken = new URLSearchParams(location.search).get('token');
    if (urlToken) {
        j('/api/auth/verify', {
            method: 'POST',
            body: JSON.stringify({ token: urlToken })
        }).then(function (resp) {
            if (resp.valid) {
                localStorage.setItem('rechat.token', urlToken);
            }
        });
        history.replaceState(null, '', location.pathname + location.hash);
    }
} else {
    // Show login, ensure token input can submit on Enter
    document.getElementById('tokenInput').addEventListener('keydown', function (e) {
        if (e.key === 'Enter') doLogin();
    });
    document.getElementById('tokenInput').focus();
}

setInterval(function () {
    var hash = location.hash || '#dashboard';
    if (hash === '#dashboard') refreshDashboard();
}, 5000);
