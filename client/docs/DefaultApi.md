# \DefaultApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cats_get**](DefaultApi.md#cats_get) | **GET** /cats | Get all cats
[**cats_id_delete**](DefaultApi.md#cats_id_delete) | **DELETE** /cats/{id} | Delete a cat by ID
[**cats_id_get**](DefaultApi.md#cats_id_get) | **GET** /cats/{id} | Get a cat by ID
[**cats_id_put**](DefaultApi.md#cats_id_put) | **PUT** /cats/{id} | Update a cat by ID
[**cats_post**](DefaultApi.md#cats_post) | **POST** /cats | Create a new cat
[**dogs_get**](DefaultApi.md#dogs_get) | **GET** /dogs | Get all dogs
[**dogs_id_delete**](DefaultApi.md#dogs_id_delete) | **DELETE** /dogs/{id} | Delete a dog by ID
[**dogs_id_get**](DefaultApi.md#dogs_id_get) | **GET** /dogs/{id} | Get a dog by ID
[**dogs_id_put**](DefaultApi.md#dogs_id_put) | **PUT** /dogs/{id} | Update a dog by ID
[**dogs_post**](DefaultApi.md#dogs_post) | **POST** /dogs | Create a new dog



## cats_get

> serde_json::Value cats_get()
Get all cats

### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cats_id_delete

> cats_id_delete(id)
Delete a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cats_id_get

> crate::models::Cat cats_id_get(id)
Get a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |

### Return type

[**crate::models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cats_id_put

> crate::models::Cat cats_id_put(id, cat)
Update a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |
**cat** | [**Cat**](Cat.md) |  | [required] |

### Return type

[**crate::models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## cats_post

> crate::models::Cat cats_post(cat)
Create a new cat

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cat** | [**Cat**](Cat.md) |  | [required] |

### Return type

[**crate::models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dogs_get

> serde_json::Value dogs_get()
Get all dogs

### Parameters

This endpoint does not need any parameter.

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dogs_id_delete

> dogs_id_delete(id)
Delete a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dogs_id_get

> crate::models::Dog dogs_id_get(id)
Get a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |

### Return type

[**crate::models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dogs_id_put

> crate::models::Dog dogs_id_put(id, dog)
Update a dog by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | [**serde_json::Value**](.md) |  | [required] |
**dog** | [**Dog**](Dog.md) |  | [required] |

### Return type

[**crate::models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dogs_post

> crate::models::Dog dogs_post(dog)
Create a new dog

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**dog** | [**Dog**](Dog.md) |  | [required] |

### Return type

[**crate::models::Dog**](Dog.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

