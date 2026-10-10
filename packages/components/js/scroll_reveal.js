const elements = document.querySelectorAll('[data-reveal]');
if (elements.length === 0) { return; }
// Trigger a little before the element's bottom edge fully reaches the
// viewport's bottom edge, and once ~10% of it is visible.
const observer = new IntersectionObserver((entries, obs) => {
  for (const entry of entries) {
    if (entry.isIntersecting) {
      entry.target.classList.add('is-visible');
      obs.unobserve(entry.target);
    }
  }
}, { rootMargin: '0px 0px -10% 0px', threshold: 0.1 });
elements.forEach((el) => observer.observe(el));
