import { animate, stagger } from 'animejs';
import { createIcons, AudioLines, BrainCircuit, FileText, SlidersHorizontal, Plug, ShieldCheck, Monitor, ChevronRight, Cpu, Sparkles, Check, Box, Zap, Layers, Brain, Info, Plus, Terminal, Search, ArrowUpRight, Cable, Mic, MessageCircle, CircleCheck, Network, LockKeyhole, Command, RotateCcw, X, Clock3 } from 'lucide';

export function icons() {
  createIcons({ icons: { AudioLines, BrainCircuit, FileText, SlidersHorizontal, Plug, ShieldCheck, Monitor, ChevronRight, Cpu, Sparkles, Check, Box, Zap, Layers, Brain, Info, Plus, Terminal, Search, ArrowUpRight, Cable, Mic, MessageCircle, CircleCheck, Network, LockKeyhole, Command, RotateCcw, X, Clock3 } });
}

const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
export function reveal(container) {
  if (reducedMotion.matches) return;
  const targets = container.querySelectorAll('.motion-item');
  animate(targets, { opacity: [0, 1], translateY: [10, 0], duration: 360, delay: stagger(45), ease: 'outCubic' });
}

export function setupDialogs() {
  const triggers = new WeakMap();
  document.querySelectorAll('dialog').forEach(dialog => {
    dialog.addEventListener('close', () => triggers.get(dialog)?.focus());
    dialog.addEventListener('click', event => {
      if (event.target !== dialog) return;
      const rect = dialog.getBoundingClientRect();
      if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) dialog.close();
    });
  });
  document.querySelectorAll('[data-close-dialog]').forEach(button => button.addEventListener('click', () => document.getElementById(button.dataset.closeDialog).close()));
  return (id, trigger) => {
    const dialog = document.getElementById(id);
    triggers.set(dialog, trigger);
    dialog.showModal();
    if (!reducedMotion.matches) animate(dialog, { opacity: [0, 1], translateY: [12, 0], scale: [.98, 1], duration: 240, ease: 'outCubic' });
  };
}
