document.addEventListener('DOMContentLoaded', () => {
    const btn = document.getElementById('actionBtn');
    const msg = document.getElementById('statusMsg');
    
    btn.addEventListener('click', () => {
        // Multi-line terminal output
        msg.innerHTML = "> Executing payload...<br>> Checking thread pool... OK.<br>> All systems nominal.";
        msg.style.color = "#4caf50"; // Bright green text
        
        // Change button appearance to show it worked
        btn.textContent = "Diagnostic Complete";
        btn.style.backgroundColor = "#2e7d32";
        btn.style.color = "white";
        btn.style.transform = "none";
        btn.style.boxShadow = "none";
        btn.disabled = true;
    });
});