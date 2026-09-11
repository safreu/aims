import { useEffect, useRef } from "react";
import "./QrScanner.css";
import {
  BrowserCodeReader,
  BrowserQRCodeReader,
  type IScannerControls,
} from "@zxing/browser";

type Props = {
  paused: boolean;
  onScan: (value: string) => void;
  onError: (error: Error) => void;
};

export function QrScanner({ paused, onScan, onError }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const controlsRef = useRef<IScannerControls | null>(null);

  const pauseRef = useRef(paused);
  const onScanRef = useRef(onScan);
  const onErrorRef = useRef(onError);

  useEffect(() => {
    pauseRef.current = paused;
  }, [paused]);

  useEffect(() => {
    onScanRef.current = onScan;
  }, [onScan]);

  useEffect(() => {
    onErrorRef.current = onError;
  }, [onError]);

  useEffect(() => {
    const video = videoRef.current;

    if (video == null) return;

    const reader = new BrowserQRCodeReader();

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
          if (result !== undefined && !pauseRef.current) {
            onScanRef.current(result.getText());
          }
        },
      )
      .then((controls) => {
        controlsRef.current = controls;
      })
      .catch((error: unknown) => {
        if (error instanceof Error) {
          onErrorRef.current(error);
        } else {
          onErrorRef.current(new Error("Failed to start QR scanner"));
        }
      });
    return () => {
      controlsRef.current?.stop();
      controlsRef.current = null;

      BrowserCodeReader.releaseAllStreams();
    };
  }, []);

  return (
    <div className="qr-scanner">
      <video ref={videoRef} className="qr-scanner__video" muted playsInline />
    </div>
  );
}
