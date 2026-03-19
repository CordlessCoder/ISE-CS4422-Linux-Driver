// ==================== Auth Management ====================
function getToken() {
    return localStorage.getItem('authToken');
}

function setToken(token, username) {
    localStorage.setItem('authToken', token);
    localStorage.setItem('username', username);
}

function clearToken() {
    localStorage.removeItem('authToken');
    localStorage.removeItem('username');
}

function isLoggedIn() {
    return getToken() !== null;
}

function showAuthScreen() {
    document.getElementById('authScreen').classList.remove('hidden');
    document.getElementById('appScreen').classList.add('hidden');
}

function showAppScreen() {
    document.getElementById('authScreen').classList.add('hidden');
    document.getElementById('appScreen').classList.remove('hidden');
    document.getElementById('currentUser').textContent = localStorage.getItem('username');
}

function switchTab(tab) {
    document.getElementById('loginTab').classList.toggle('hidden', tab !== 'login');
    document.getElementById('registerTab').classList.toggle('hidden', tab !== 'register');

    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');
}

// ==================== Status Display ====================
function showStatus(elementId, message, type) {
    const element = document.getElementById(elementId);
    element.textContent = message;
    element.className = `status show ${type}`;
}

function hideStatus(elementId) {
    document.getElementById(elementId).className = 'status';
}

// ==================== Auth Endpoints ====================
async function register() {
    const username = document.getElementById('registerUsername').value.trim();
    const password = document.getElementById('registerPassword').value;

    if (!username || !password) {
        showStatus('registerStatus', 'Fill in all fields', 'error');
        return;
    }

    showStatus('registerStatus', 'Registering...', 'loading');

    try {
        const res = await fetch('/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });

        const data = await res.json();

        if (res.ok) {
            showStatus('registerStatus', 'Registered! Now login.', 'success');
            setTimeout(() => switchTab('login'), 2000);
            document.getElementById('registerUsername').value = '';
            document.getElementById('registerPassword').value = '';
        } else {
            showStatus('registerStatus', data.error || 'Registration failed', 'error');
        }
    } catch (err) {
        showStatus('registerStatus', 'Network error', 'error');
    }
}

async function login() {
    const username = document.getElementById('loginUsername').value.trim();
    const password = document.getElementById('loginPassword').value;

    if (!username || !password) {
        showStatus('loginStatus', 'Fill in all fields', 'error');
        return;
    }

    showStatus('loginStatus', 'Logging in...', 'loading');

    try {
        const res = await fetch('/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });

        const data = await res.json();

        if (res.ok) {
            setToken(data.token, data.username);
            showStatus('loginStatus', 'Success!', 'success');
            setTimeout(showAppScreen, 500);
            document.getElementById('loginUsername').value = '';
            document.getElementById('loginPassword').value = '';
        } else {
            showStatus('loginStatus', data.error || 'Login failed', 'error');
        }
    } catch (err) {
        showStatus('loginStatus', 'Network error', 'error');
    }
}

async function logout() {
    const token = getToken();
    try {
        await fetch('/logout', {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}` }
        });
    } catch (err) {
        // ignore errors
    }
    clearToken();
    showAuthScreen();
}

// ==================== Vault Endpoints ====================
function getAuthHeaders() {
    return {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${getToken()}`
    };
}

async function createVault() {
    const name = document.getElementById('createName').value.trim();
    const password = document.getElementById('createPassword').value;

    if (!name || !password) {
        showStatus('createStatus', 'Fill in all fields', 'error');
        return;
    }

    showStatus('createStatus', 'Creating...', 'loading');

    try {
        const res = await fetch('/create', {
            method: 'POST',
            headers: getAuthHeaders(),
            body: JSON.stringify({ name, password })
        });

        const data = await res.json();

        if (res.ok) {
            showStatus('createStatus', data.message, 'success');
            document.getElementById('createName').value = '';
            document.getElementById('createPassword').value = '';
        } else {
            showStatus('createStatus', data.error || 'Failed', 'error');
        }
    } catch (err) {
        showStatus('createStatus', 'Network error', 'error');
    }
}

async function unlockVault() {
    const name = document.getElementById('unlockName').value.trim();
    const password = document.getElementById('unlockPassword').value;

    if (!name || !password) {
        showStatus('unlockStatus', 'Fill in all fields', 'error');
        return;
    }

    showStatus('unlockStatus', 'Unlocking...', 'loading');

    try {
        const res = await fetch('/unlock', {
            method: 'POST',
            headers: getAuthHeaders(),
            body: JSON.stringify({ name, password })
        });

        const data = await res.json();

        if (res.ok) {
            showStatus('unlockStatus', 'Success', 'success');
            const output = document.getElementById('unlockOutput');
            output.textContent = data.data;
            output.classList.remove('empty');
        } else {
            showStatus('unlockStatus', data.error || 'Failed', 'error');
            document.getElementById('unlockOutput').className = 'output empty';
            document.getElementById('unlockOutput').textContent = 'data will appear here';
        }
    } catch (err) {
        showStatus('unlockStatus', 'Network error', 'error');
    }
}

async function saveVault() {
    const name = document.getElementById('saveName').value.trim();
    const password = document.getElementById('savePassword').value;
    const data = document.getElementById('saveData').value;

    if (!name || !password || !data) {
        showStatus('saveStatus', 'Fill in all fields', 'error');
        return;
    }

    showStatus('saveStatus', 'Saving...', 'loading');

    try {
        const res = await fetch('/save', {
            method: 'POST',
            headers: getAuthHeaders(),
            body: JSON.stringify({ name, password, data })
        });

        const result = await res.json();

        if (res.ok) {
            showStatus('saveStatus', result.message, 'success');
            document.getElementById('saveData').value = '';
        } else {
            showStatus('saveStatus', result.error || 'Failed', 'error');
        }
    } catch (err) {
        showStatus('saveStatus', 'Network error', 'error');
    }
}

// ==================== Init ====================
window.addEventListener('load', () => {
    if (isLoggedIn()) {
        showAppScreen();
    } else {
        showAuthScreen();
    }
});
