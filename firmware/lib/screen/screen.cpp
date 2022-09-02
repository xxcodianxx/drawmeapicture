#include <screen.h>

MCUFRIEND_kbv tft;
TouchScreen ts(XP, YP, XM, YM, TFT_RESISTANCE);

TSPoint getTSPoint() {
    TSPoint tsp = ts.getPoint();
    
    pinMode(XM, OUTPUT);
    pinMode(YP, OUTPUT);

    return tsp;
}

void touchToScreen(TSPoint* p) {
    // int16_t px = map(p->y, TS_MINX, TS_MAXX, tft.width(), 0);
    // int16_t py = map(p->x, TS_MINY, TS_MAXY, tft.height(), 0);
    // p->x = px;
    // p->y = py;

    p->x = map(p->x, TS_MINX, TS_MAXX, 0, tft.width());
    p->y = map(p->y, TS_MINY, TS_MAXY, tft.height(), 0);
}

bool isPointDown(TSPoint* p) {
    return ((p->z >= TS_MINP) && (p->z <= TS_MAXP));
}

void screen_init() {
    tft.reset();
    tft.begin(tft.readID());
    tft.setRotation(0);
    
    tft.fillScreen(TFT_BLACK);
}