if (!window.__noCtxMenu) {
    window.__noCtxMenu = true;
    document.addEventListener('contextmenu', (e) => {
        if (e.target.closest('input, textarea, [data-allow-contextmenu]')) return;
        e.preventDefault();
    });
}
