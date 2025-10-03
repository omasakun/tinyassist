import pc from "picocolors";

export function info(message: string): void {
  console.log(pc.blue(message));
}

export function success(message: string): void {
  console.log(pc.green(message));
}

export function error(message: string): void {
  console.error(pc.red(message));
}

export function warning(message: string): void {
  console.log(pc.yellow(message));
}
