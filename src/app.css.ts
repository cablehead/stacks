// Will need:
// https://github.com/Qix-/color#readme

import { createTheme, globalStyle, style } from "@vanilla-extract/css";

export const [darkThemeClass, vars] = createTheme({
  backgroundColor: "#1F1F1F",
  backgroundColorTransparent: "#1F1F1FEF",
  headerColor: "#fffffe",
  textColor: "#a7a9be",
  buttonBackgroundColor: "#ff8906",
  buttonTextColor: "#fffffe",
  altBackgroundColor: "#fffffe",
  altTextColor: "black",
  backgroundColorSelected: "#3A3A3A",
  backgroundColorButton: "#2A2A2A",
  backgroundColorHover: "#2A2A2A",
  altTertiaryColor: "#e53170",
  borderColor: "#333",
});

export const lightThemeClass = createTheme(vars, {
  backgroundColor: "#F0F0F0",
  backgroundColorTransparent: "#F0F0F0EF",
  headerColor: "#272343",
  textColor: "#2d334a",
  buttonBackgroundColor: "#ffd803",
  buttonTextColor: "#272343",
  altBackgroundColor: "#fffffe",
  altTextColor: "#272343",
  backgroundColorSelected: "#D1D1D1",
  backgroundColorButton: "#E0E0E0",
  backgroundColorHover: "#E0E0E0",
  altTertiaryColor: "#bae8e8",
  borderColor: "#ddd",
});

globalStyle("html, body", {
  fontFamily: "ui-monospace",
  backgroundColor: "transparent",
  padding: "10px",
  letterSpacing: "-0.02ch",
  fontSize: "16px",
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
  boxShadow: "0 -1px 3px rgba(0, 0, 0, 0.1)",
  fontSize: "0.8rem",
  backgroundColor: vars.backgroundColor,
  padding: "1ch",
  paddingLeft: "2ch",
  paddingRight: "2ch",
  justifyContent: "space-between",
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
