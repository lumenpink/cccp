// CCCP Soviet Documentation Site Scripts

document.addEventListener('DOMContentLoaded', () => {
  // Remove any polyglot shell shebang text nodes from DOM
  if (document.body) {
    Array.from(document.body.childNodes).forEach((node) => {
      if (node.nodeType === Node.TEXT_NODE && (node.textContent.includes('#!/bin/sh') || node.textContent.includes('<<\\EOF'))) {
        node.remove();
      }
    });
  }

  // Add copy-to-clipboard buttons to all code blocks
  document.querySelectorAll('pre').forEach((block) => {
    const copyButton = document.createElement('button');
    copyButton.className = 'copy-btn';
    copyButton.type = 'button';
    copyButton.setAttribute('aria-label', 'Copy code to clipboard');
    copyButton.innerHTML = '★ COPY';

    copyButton.addEventListener('click', async () => {
      const code = block.querySelector('code')?.innerText || block.innerText;
      try {
        await navigator.clipboard.writeText(code);
        copyButton.innerHTML = '✔ COPIED!';
        copyButton.classList.add('copied');
        setTimeout(() => {
          copyButton.innerHTML = '★ COPY';
          copyButton.classList.remove('copied');
        }, 2000);
      } catch (err) {
        console.error('Clipboard copy failed:', err);
      }
    });

    block.style.position = 'relative';
    block.appendChild(copyButton);
  });

  // Filter commands on commands page
  const searchInput = document.getElementById('command-search');
  if (searchInput) {
    searchInput.addEventListener('input', (e) => {
      const query = e.target.value.toLowerCase().trim();
      const commandSections = document.querySelectorAll('.command-item');
      commandSections.forEach((section) => {
        const text = section.innerText.toLowerCase();
        if (!query || text.includes(query)) {
          section.style.display = '';
        } else {
          section.style.display = 'none';
        }
      });
    });
  }
});
