#pragma once

#include <Adafruit_GFX.h>
#include <TouchScreen.h>
#include <MCUFRIEND_kbv.h>

// LCD pinout
#define LCD_CS    A3
#define LCD_CD    A2
#define LCD_WR    A1
#define LCD_RD    A0
#define LCD_RESET A4

// Touch pinout
#define YP A3
#define XM A2
#define YM 9
#define XP 8

#define TFT_RESISTANCE 300

// Touch boundaries (offset)
#define TS_MINX 113
#define TS_MAXX 906
#define TS_MINY 94
#define TS_MAXY 966

// Touch pressure
#define TS_MINP 10
#define TS_MAXP 1000

extern MCUFRIEND_kbv tft;
extern TouchScreen ts;

TSPoint getTSPoint(TouchScreen &ts);
void touchLtoP(MCUFRIEND_kbv* tft, TSPoint* p);
bool isPointDown(TSPoint* p);
void screen_init();