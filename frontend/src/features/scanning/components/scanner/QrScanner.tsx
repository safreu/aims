import { useEffect, useRef } from "react";
import "./QrScanner.css";
import { BrowserQRCodeReader, type IScannerControls } from "@zxing/browser";

type Props = {
  paused: boolean;
  onScan: (value: string) => void;
  onError: (error: Error) => void;
};

export function QrScanner({ paused, onScan, onError }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const controlsRef = useRef<IScannerControls | null>(null);
  const pauseRef = useRef(paused);

  useEffect(() => {
    pauseRef.current = paused;
  }, [paused]);

  useEffect(() => {
    const video = videoRef.current;

    if (video == null) return;

    const reader = new BrowserQRCodeReader();

    let disposed = false;

    void reader
      .decodeFromConstraints(
        {
          video: {
            facingMode: {
              ideal: "environment",
            },
          },
        },
        video,
        (result) => {
          if (result !== undefined && !pauseRef.current && !disposed) {
            onScan(result.getText());
          }
        },
      )
      .then((controls) => {
        if (disposed) {
          controls.stop();
          return;
        }

        controlsRef.current = controls;
      })
      .catch((error: unknown) => {
        if (disposed) return;

        if (error instanceof Error) {
          onError(error);
        } else {
          onError(new Error("Failed to start QR scanner"));
        }
      });
    return () => {
      disposed = true;

      controlsRef.current?.stop();
      controlsRef.current = null;

      const stream = video.srcObject;

      if (stream instanceof MediaStream) {
        for (const track of stream.getTracks()) track.stop();
      }

      video.srcObject = null;
    };
  }, [onScan, onError]);

  return (
    <div className="qr-scanner">
      <video ref={videoRef} className="qr-scanner__video" muted playsInline />
    </div>
  );
}
