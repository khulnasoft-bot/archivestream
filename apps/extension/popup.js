document.addEventListener('DOMContentLoaded', async () => {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    const urlPreview = document.getElementById('urlPreview');
    const snapBtn = document.getElementById('snapBtn');
    const status = document.getElementById('status');
    const instanceInput = document.getElementById('instanceUrl');

    if (tab) {
        urlPreview.textContent = tab.url;
    }

    // Load saved instance URL
    chrome.storage.local.get(['instanceUrl'], (result) => {
        if (result.instanceUrl) {
            instanceInput.value = result.instanceUrl;
        }
    });

    instanceInput.addEventListener('change', () => {
        chrome.storage.local.set({ instanceUrl: instanceInput.value });
    });

    snapBtn.addEventListener('click', async () => {
        const instanceUrl = instanceInput.value.replace(/\/$/, '');
        snapBtn.disabled = true;
        snapBtn.textContent = 'Snapping...';

        try {
            const response = await fetch(`${instanceUrl}/crawl`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url: tab.url })
            });

            if (response.ok) {
                status.style.display = 'block';
                setTimeout(() => window.close(), 2000);
            } else {
                alert('Failed to queue: ' + response.statusText);
            }
        } catch (e) {
            alert('Error connecting to instance: ' + e.message);
        } finally {
            snapBtn.disabled = false;
            snapBtn.textContent = 'Snap Current Page';
        }
    });
});
