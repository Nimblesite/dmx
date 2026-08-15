window.mermaid.initialize({
  startOnLoad: true,
  securityLevel: "strict",
  theme: document.documentElement.dataset.theme === "dark" ? "dark" : "neutral",
  flowchart: {
    curve: "basis",
    htmlLabels: false,
  },
});
