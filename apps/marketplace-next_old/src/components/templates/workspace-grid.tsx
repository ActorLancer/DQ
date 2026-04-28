import { ReactNode } from "react";

export function WorkspaceGrid({
  left,
  center,
  right,
}: {
  left?: ReactNode;
  center: ReactNode;
  right?: ReactNode;
}) {
  if (left && right) {
    return (
      <div className="grid h-full grid-cols-[300px_1fr_360px] gap-3 overflow-hidden">
        <section className="h-full overflow-auto rounded-lg panel">{left}</section>
        <section className="h-full overflow-auto rounded-lg panel">{center}</section>
        <section className="h-full overflow-auto rounded-lg panel">{right}</section>
      </div>
    );
  }
  if (left) {
    return (
      <div className="grid h-full grid-cols-[300px_1fr] gap-3 overflow-hidden">
        <section className="h-full overflow-auto rounded-lg panel">{left}</section>
        <section className="h-full overflow-auto rounded-lg panel">{center}</section>
      </div>
    );
  }
  if (right) {
    return (
      <div className="grid h-full grid-cols-[1fr_360px] gap-3 overflow-hidden">
        <section className="h-full overflow-auto rounded-lg panel">{center}</section>
        <section className="h-full overflow-auto rounded-lg panel">{right}</section>
      </div>
    );
  }
  return <section className="h-full overflow-auto rounded-lg panel">{center}</section>;
}
