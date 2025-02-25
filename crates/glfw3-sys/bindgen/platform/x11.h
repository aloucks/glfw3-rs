typedef void Display;
typedef void* RRCrtc;
typedef void* RROutput;
typedef void* Window;

typedef void* GLXContext;
typedef void* GLXWindow;

#define GLFW_INCLUDE_NONE
#define GLFW_NATIVE_INCLUDE_NONE
#define GLFW_EXPOSE_NATIVE_X11
#define GLFW_EXPOSE_NATIVE_GLX

#include "../../vendor/glfw/include/GLFW/glfw3.h"
#include "../../vendor/glfw/include/GLFW/glfw3native.h"