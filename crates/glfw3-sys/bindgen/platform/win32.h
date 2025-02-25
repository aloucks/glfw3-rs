typedef void* HWND;
typedef void* HGLRC;

#define GLFW_INCLUDE_NONE
#define GLFW_NATIVE_INCLUDE_NONE
#define GLFW_EXPOSE_NATIVE_WIN32
#define GLFW_EXPOSE_NATIVE_WGL

#include "../../vendor/glfw/include/GLFW/glfw3.h"
#include "../../vendor/glfw/include/GLFW/glfw3native.h"