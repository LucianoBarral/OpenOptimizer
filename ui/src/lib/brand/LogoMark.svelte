<script lang="ts">
  // Logo real (PNG sem o ponteiro) + ponteiro redesenhado em SVG por cima, para poder girar.
  // Geometria medida na logo original por scripts/brand-assets.ps1 (needle.json, espaço 1024x1024).
  import dial from '../../assets/brand/logo-dial.png';
  import needle from '../../assets/brand/needle.json';

  type Mode = 'intro' | 'loop' | 'static';
  let { size = 120, mode = 'static' }: { size?: number; mode?: Mode } = $props();

  const len = Math.hypot(needle.tipX - needle.hubX, needle.tipY - needle.hubY);
  const rest = (Math.atan2(needle.tipY - needle.hubY, needle.tipX - needle.hubX) * 180) / Math.PI;
  const id = $props.id();
</script>

<div class="logo {mode}" style="--s:{size}px; --rest:{rest}deg; --mask:url({dial})" aria-hidden="true">
  <img src={dial} alt="" draggable="false" />
  <svg viewBox="0 0 1024 1024">
    <defs>
      <linearGradient id="n-{id}" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0" stop-color="#ffffff" />
        <stop offset="1" stop-color="#cfd5db" />
      </linearGradient>
    </defs>
    <g transform="translate({needle.hubX} {needle.hubY})">
      <g class="needle">
        <polygon points="0,-13 {len - 8},-2.5 {len},0 {len - 8},2.5 0,13" fill="url(#n-{id})" />
        <circle r={needle.hubR} fill="url(#n-{id})" />
      </g>
    </g>
  </svg>
  <span class="sheen"><i></i></span>
</div>

<style>
  .logo {
    position: relative;
    width: var(--s);
    height: var(--s);
    flex: none;
  }
  img,
  svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  img {
    user-select: none;
  }
  .needle {
    transform: rotate(var(--rest));
  }

  /* Faixa de luz recortada pelo formato da logo (máscara = a própria logo). */
  .sheen {
    position: absolute;
    inset: 0;
    overflow: hidden;
    -webkit-mask: var(--mask) center / contain no-repeat;
    mask: var(--mask) center / contain no-repeat;
    pointer-events: none;
  }
  .sheen i {
    position: absolute;
    top: -20%;
    bottom: -20%;
    width: 38%;
    left: -45%;
    background: linear-gradient(100deg, transparent, rgba(255, 255, 255, 0.55), transparent);
    opacity: 0;
  }

  /* Abertura: a logo surge, o ponteiro "acelera" do zero ao fim da escala e assenta. */
  .intro img {
    animation: appear 0.7s var(--ease) both;
  }
  .intro svg {
    animation: fade 0.4s var(--ease) 0.15s both;
  }
  .intro .needle {
    animation: rev 1.25s cubic-bezier(0.3, 0.7, 0.2, 1) 0.15s both;
  }
  .intro .sheen i {
    animation: sweep 0.9s var(--ease) 0.75s both;
  }

  /* Carregando: o ponteiro varre a escala em loop; a luz passa de tempos em tempos. */
  .loop .needle {
    animation: work 1.1s ease-in-out infinite alternate;
  }
  .loop .sheen i {
    animation: sweep 2.4s var(--ease) infinite;
  }

  @keyframes appear {
    from {
      opacity: 0;
      transform: scale(0.94);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes rev {
    0% {
      transform: rotate(-168deg);
    }
    62% {
      transform: rotate(calc(var(--rest) + 9deg));
    }
    82% {
      transform: rotate(calc(var(--rest) - 4deg));
    }
    100% {
      transform: rotate(var(--rest));
    }
  }
  @keyframes work {
    from {
      transform: rotate(-150deg);
    }
    to {
      transform: rotate(calc(var(--rest) + 6deg));
    }
  }
  @keyframes sweep {
    0% {
      opacity: 0;
      transform: translateX(0);
    }
    15% {
      opacity: 1;
    }
    60%,
    100% {
      opacity: 0;
      transform: translateX(400%);
    }
  }
</style>
