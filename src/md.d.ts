// Markdown files imported with `?raw` (e.g. the welcome page) are plain strings.
declare module "*.md?raw" {
  const content: string;
  export default content;
}
