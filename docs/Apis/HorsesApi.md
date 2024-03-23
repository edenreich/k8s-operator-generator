# HorsesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createHorse**](HorsesApi.md#createHorse) | **POST** /horses | Create a new horse |
| [**deleteHorseById**](HorsesApi.md#deleteHorseById) | **DELETE** /horses/{id} | Delete a horse by ID |
| [**getHorseById**](HorsesApi.md#getHorseById) | **GET** /horses/{id} | Get a horse by ID |
| [**getHorses**](HorsesApi.md#getHorses) | **GET** /horses | Get all horses |
| [**updateHorseById**](HorsesApi.md#updateHorseById) | **PUT** /horses/{id} | Update a horse by ID |


<a name="createHorse"></a>
# **createHorse**
> Horse createHorse(Horse)

Create a new horse

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **Horse** | [**Horse**](../Models/Horse.md)|  | |

### Return type

[**Horse**](../Models/Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="deleteHorseById"></a>
# **deleteHorseById**
> deleteHorseById(id)

Delete a horse by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |

### Return type

null (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

<a name="getHorseById"></a>
# **getHorseById**
> Horse getHorseById(id)

Get a horse by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |

### Return type

[**Horse**](../Models/Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getHorses"></a>
# **getHorses**
> List getHorses()

Get all horses

### Parameters
This endpoint does not need any parameter.

### Return type

[**List**](../Models/Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="updateHorseById"></a>
# **updateHorseById**
> Horse updateHorseById(id, Horse)

Update a horse by ID

### Parameters

|Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **id** | **String**|  | [default to null] |
| **Horse** | [**Horse**](../Models/Horse.md)|  | |

### Return type

[**Horse**](../Models/Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

