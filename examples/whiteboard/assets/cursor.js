document.addEventListener("DOMContentLoaded", () => {
    // Inject global cursor styles
    const style = document.createElement("style");
    style.innerHTML = `
        * {
            cursor: none !important;
        }
        #global-cursor {
            position: fixed;
            top: 0;
            left: 0;
            margin-top: -3px;
            margin-left: -12.5px;
            pointer-events: none;
            z-index: 99999;
            opacity: 0;
            transform-origin: 12.5px 3px;
            will-change: transform;
        }
    `;
    document.head.appendChild(style);

    // Create cursor element
    const cursor = document.createElement("div");
    cursor.id = "global-cursor";
    cursor.innerHTML = `
    <svg xmlns="http://www.w3.org/2000/svg" width="25" height="27" viewBox="0 0 50 54" fill="none">
      <g filter="url(#filter0_d_global)">
        <path d="M42.6817 41.1495L27.5103 6.79925C26.7269 5.02557 24.2082 5.02558 23.3927 6.79925L7.59814 41.1495C6.75833 42.9759 8.52712 44.8902 10.4125 44.1954L24.3757 39.0496C24.8829 38.8627 25.4385 38.8627 25.9422 39.0496L39.8121 44.1954C41.6849 44.8902 43.4884 42.9759 42.6817 41.1495Z" fill="black" />
        <path d="M43.7146 40.6933L28.5431 6.34306C27.3556 3.65428 23.5772 3.69516 22.3668 6.32755L6.57226 40.6778C5.3134 43.4156 7.97238 46.298 10.803 45.2549L24.7662 40.109C25.0221 40.0147 25.2999 40.0156 25.5494 40.1082L39.4193 45.254C42.2261 46.2953 44.9254 43.4347 43.7146 40.6933Z" stroke="white" stroke-width="2.25825" />
      </g>
      <defs>
        <filter id="filter0_d_global" x="0.602397" y="0.952444" width="49.0584" height="52.428" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
          <feFlood flood-opacity="0" result="BackgroundImageFix" />
          <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha" />
          <feOffset dy="2.25825" />
          <feGaussianBlur stdDeviation="2.25825" />
          <feComposite in2="hardAlpha" operator="out" />
          <feColorMatrix type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.08 0" />
          <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow_global" />
          <feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow_global" result="shape" />
        </filter>
      </defs>
    </svg>
    `;
    document.body.appendChild(cursor);

    let lastTime = performance.now();
    let lastX = 0;
    let lastY = 0;
    let prevAngle = 0;
    let accRotation = 0;
    let scaleTimeout = null;

    window.addEventListener("pointermove", (e) => {
        if (e.pointerType === "touch") return; // don't track touch

        const now = performance.now();
        const dt = now - lastTime;
        let vx = 0;
        let vy = 0;
        
        if (dt > 0) {
            vx = (e.clientX - lastX) / dt;
            vy = (e.clientY - lastY) / dt;
        }

        lastTime = now;
        lastX = e.clientX;
        lastY = e.clientY;

        const speed = Math.sqrt(vx * vx + vy * vy);
        let scale = 1.0;

        if (speed > 0.1) {
            const currentAngle = Math.atan2(vy, vx) * (180 / Math.PI) + 90;
            let angleDiff = currentAngle - prevAngle;
            if (angleDiff > 180) angleDiff -= 360;
            if (angleDiff < -180) angleDiff += 360;
            accRotation += angleDiff;
            prevAngle = currentAngle;
            scale = 0.95;
        }

        if (window.gsap) {
            gsap.to(cursor, {
                x: e.clientX,
                y: e.clientY,
                rotation: accRotation,
                scale: scale,
                opacity: 1,
                duration: 0.15,
                ease: "power2.out",
                overwrite: "auto"
            });
            
            if (scaleTimeout) clearTimeout(scaleTimeout);
            scaleTimeout = setTimeout(() => {
                gsap.to(cursor, { scale: 1, duration: 0.15, overwrite: "auto" });
            }, 150);
        }
    }, { passive: true });
});
