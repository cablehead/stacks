// Will need:
// https://github.com/Qix-/color#readme

import { createTheme, globalStyle, style } from "@vanilla-extract/css";

export const [darkThemeClass, vars] = createTheme({
  textColor: "#a7a9be",
  backgroundColor: "#1F1F1F",
  backgroundColorTransparent: "#1F1F1FEF",
  backgroundColorSelected: "#4A4A4A",
  backgroundColorButton: "#333333",
  backgroundColorHover: "#333333",
  borderColor: "#333",
  shadowColor: "rgba(100, 100, 100, 0.2)",
});

export const lightThemeClass = createTheme(vars, {
  textColor: "#2d334a",
  backgroundColor: "#FFFFFF",
  backgroundColorTransparent: "#FFFFFFEF",
  backgroundColorSelected: "#D1D1D1",
  backgroundColorButton: "#E0E0E0",
  backgroundColorHover: "#E0E0E0",
  borderColor: "#ddd",
  shadowColor: "rgba(0, 0, 0, 0.1)",
});

globalStyle("html, body", {
  fontFamily: "ui-monospace",
  backgroundColor: "transparent",
  padding: "10px",
  letterSpacing: "-0.02ch",
  fontSize: "14px",
  height: "100%",
  overflow: "hidden",
});

globalStyle("main", {
  color: vars.textColor,
  backgroundColor: vars.backgroundColorTransparent,
  borderRadius: "10px",
  boxShadow: "0 0 10px rgba(0, 0, 0, 0.5)",
  width: "100%",
  height: "100%",
  overflow: "auto",
  display: "flex",
  flexDirection: "column",
});

globalStyle(".hoverable", {
  borderRadius: "4px",
  padding: "4px",
});

globalStyle(".hoverable:hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle("input", {
  border: "none",
  width: "100%",
  outline: "none",
});

globalStyle("textarea", {
  padding: "0.5ch",
  border: "1px solid #ccc",
  borderRadius: "0.5ch",
  resize: "none",
});

globalStyle("textarea:hover", {
  borderColor: "#999",
});

globalStyle("textarea:focus", {
  outline: "none",
  borderColor: "#aaa",
  boxShadow: "0 0 2px #aaa",
});

globalStyle(".terserow:hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle(".terserow.selected", {
  backgroundColor: vars.backgroundColorSelected,
});

export const footer = style({
  display: "flex",
  alignItems: "center",
  height: "5ch",
  boxShadow: "0 -1px 3px " + vars.shadowColor,
  fontSize: "0.9rem",
  backgroundColor: vars.backgroundColor,
  padding: "1ch",
  paddingLeft: "2ch",
  paddingRight: "2ch",
  justifyContent: "space-between",
});

export const overlay = style({
  boxShadow: "0 0 5px " + vars.shadowColor,
  backgroundColor: vars.backgroundColorTransparent,
});

export const borderRight = style({
  borderRightWidth: "1px",
  borderRightStyle: "solid",
  borderRightColor: vars.borderColor,
});

export const borderBottom = style({
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: vars.borderColor,
});

export const iconStyle = style({
  display: "inline-block",
  height: "1.5em",
  textAlign: "center",
  backgroundColor: vars.backgroundColorButton,
  borderRadius: "5px",
  paddingLeft: "1ch",
  paddingRight: "1ch",
});
