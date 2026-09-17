import {
  BrowserCodeReader,
  BrowserQRCodeReader,
  type IScannerControls,
} from "@zxing/browser";
import { useEffect, useRef } from "react";

import styles from "./QrScanner.module.css";

type Props = {
  paused: boolean;
  onScan: (value: string) => void;
  onError: (error: Error) => void;
};

export function QrScanner({ paused, onScan, onError }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const controlsRef = useRef<IScannerControls | null>(null);

  const pausedRef = useRef(paused);
  const onScanRef = useRef(onScan);
  const onErrorRef = useRef(onError);

  useEffect(() => {
    pausedRef.current = paused;
  }, [paused]);

  useEffect(() => {
    onScanRef.current = onScan;
  }, [onScan]);

  useEffect(() => {
    onErrorRef.current = onError;
  }, [onError]);

  useEffect(() => {
    const video = videoRef.current;

    if (video === null) return;

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
          if (result !== undefined && !pausedRef.current) {
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
          return;
        }

        onErrorRef.current(new Error("Failed to start QR scanner"));
      });

    return () => {
      controlsRef.current?.stop();
      controlsRef.current = null;

      BrowserCodeReader.releaseAllStreams();
    };
  }, []);

  return (
    <div className={styles.scanner}>
      <video ref={videoRef} className={styles.video} muted playsInline />
    </div>
  );
}
