// Will need:
// https://github.com/Qix-/color#readme

import { createTheme, globalStyle, style } from "@vanilla-extract/css";

export const [darkThemeClass, vars] = createTheme({
  textColor: "#bebebe",
  backgroundColor: "#1F1F1F",
  backgroundColorTransparent: "#1F1F1FEF",
  backgroundColorSelected: "#4A4A4A",
  backgroundColorButton: "#4A4A4A",
  backgroundColorHover: "#333333",
  borderColor: "#3e3e3e",
  shadowColor: "rgba(102, 102, 102, 0.4)",
});

export const lightThemeClass = createTheme(vars, {
  textColor: "#2d334a",
  backgroundColor: "#FFFFFF",
  backgroundColorTransparent: "#FFFFFFEF",
  backgroundColorSelected: "#D1D1D1",
  backgroundColorButton: "#D1D1D1",
  backgroundColorHover: "#E2E2E2",
  borderColor: "#ccc",
  shadowColor: "rgba(0, 0, 0, 0.2)",
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

globalStyle("a", {
    color: vars.textColor,
    textDecoration: "underline",
    textDecorationColor: vars.borderColor,
    textUnderlineOffset: "3px",
});

globalStyle(".terserow:hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle(".terserow.hover", {
  backgroundColor: vars.backgroundColorHover,
});

globalStyle(".terserow.selected", {
  backgroundColor: vars.backgroundColorSelected,
});

export const footer = style({
  display: "flex",
  alignItems: "center",
  height: "5ch",
  boxShadow: "0 0 4px " + vars.shadowColor,
  fontSize: "0.9rem",
  backgroundColor: vars.backgroundColor,
  padding: "1ch",
  paddingLeft: "2ch",
  paddingRight: "1ch",
  justifyContent: "space-between",
});

export const overlay = style({
  boxShadow: "0 0 6px " + vars.shadowColor,
  backgroundColor: vars.backgroundColorTransparent,
});

export const card = style({
    backgroundColor: vars.backgroundColorTransparent,
    height: "100%",
    width: "auto",
    display: "flex",
    borderRadius: "1ch",
    flexDirection: "column",
});

export const borderRight = style({
  borderRightWidth: "1px",
  borderRightStyle: "solid",
  borderRightColor: vars.borderColor,
});

export const border = style({
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: vars.borderColor,
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
