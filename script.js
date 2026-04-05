document.addEventListener('DOMContentLoaded', () => {
    console.log("Rust server successfully loaded the JavaScript file.");
    
    const btn = document.getElementById('actionBtn');
    const msg = document.getElementById('statusMsg');
    
    btn.addEventListener('click', () => {
        msg.textContent = "> All systems nominal. Client-side JS operational.";
        btn.disabled = true;
        btn.style.opacity = '0.5';
    });
});