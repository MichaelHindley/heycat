import { useEffect, useRef } from "react";
import catVideo from "../../assets/video/cat_listening.webm";
import "./CatOverlay.css";

export function CatOverlay() {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current) {
      videoRef.current.play().catch(console.error);
    }
  }, []);

  return (
    <div className="cat-overlay">
      <video
        ref={videoRef}
        className="cat-overlay__video"
        src={catVideo}
        loop
        muted
        playsInline
        autoPlay
      />
    </div>
  );
}
