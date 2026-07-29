import { useEffect, useRef } from "react";

type Star = {
  x: number;
  y: number;
  r: number;
  a: number;
  tw: number;
  sp: number;
};

type Shooting = {
  x: number;
  y: number;
  len: number;
  speed: number;
  life: number;
  maxLife: number;
  angle: number;
};

/**
 * Fixed night-sky backdrop: twinkling stars + occasional shooting stars.
 */
export default function Starfield() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let raf = 0;
    let w = 0;
    let h = 0;
    let stars: Star[] = [];
    let shooters: Shooting[] = [];
    let spawnTimer = 0;

    const resize = () => {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = window.innerWidth;
      h = window.innerHeight;
      canvas.width = Math.floor(w * dpr);
      canvas.height = Math.floor(h * dpr);
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const count = Math.floor((w * h) / 9000);
      stars = Array.from({ length: count }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        r: Math.random() * 1.4 + 0.3,
        a: Math.random() * 0.5 + 0.25,
        tw: Math.random() * Math.PI * 2,
        sp: 0.008 + Math.random() * 0.02,
      }));
    };

    const spawnShooter = () => {
      shooters.push({
        x: Math.random() * w * 0.85,
        y: Math.random() * h * 0.45,
        len: 80 + Math.random() * 120,
        speed: 10 + Math.random() * 14,
        life: 0,
        maxLife: 28 + Math.random() * 18,
        angle: Math.PI / 4 + (Math.random() * 0.25 - 0.08),
      });
    };

    const draw = () => {
      ctx.clearRect(0, 0, w, h);

      for (const s of stars) {
        if (!reduceMotion) s.tw += s.sp;
        const pulse = reduceMotion ? s.a : s.a * (0.65 + 0.35 * Math.sin(s.tw));
        ctx.beginPath();
        ctx.fillStyle = `rgba(232, 234, 237, ${pulse})`;
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fill();
      }

      if (!reduceMotion) {
        spawnTimer += 1;
        if (spawnTimer > 50 + Math.random() * 90) {
          spawnTimer = 0;
          if (shooters.length < 3) spawnShooter();
        }

        shooters = shooters.filter((sh) => {
          sh.life += 1;
          const t = sh.life / sh.maxLife;
          const dx = Math.cos(sh.angle) * sh.speed;
          const dy = Math.sin(sh.angle) * sh.speed;
          sh.x += dx;
          sh.y += dy;

          const fade = t < 0.15 ? t / 0.15 : t > 0.7 ? (1 - t) / 0.3 : 1;
          const tx = sh.x - Math.cos(sh.angle) * sh.len;
          const ty = sh.y - Math.sin(sh.angle) * sh.len;

          const grad = ctx.createLinearGradient(tx, ty, sh.x, sh.y);
          grad.addColorStop(0, "rgba(200, 205, 212, 0)");
          grad.addColorStop(0.55, `rgba(220, 224, 230, ${0.35 * fade})`);
          grad.addColorStop(1, `rgba(248, 250, 252, ${0.95 * fade})`);

          ctx.strokeStyle = grad;
          ctx.lineWidth = 1.5;
          ctx.lineCap = "round";
          ctx.beginPath();
          ctx.moveTo(tx, ty);
          ctx.lineTo(sh.x, sh.y);
          ctx.stroke();

          ctx.beginPath();
          ctx.fillStyle = `rgba(255, 255, 255, ${fade})`;
          ctx.arc(sh.x, sh.y, 1.6, 0, Math.PI * 2);
          ctx.fill();

          return sh.life < sh.maxLife && sh.x < w + 40 && sh.y < h + 40;
        });
      }

      raf = requestAnimationFrame(draw);
    };

    resize();
    draw();
    window.addEventListener("resize", resize);

    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="pointer-events-none fixed inset-0 z-0"
      aria-hidden
    />
  );
}
