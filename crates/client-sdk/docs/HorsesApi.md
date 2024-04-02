# \HorsesApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_horse**](HorsesApi.md#create_horse) | **POST** /horses | Create a new horse
[**delete_horse_by_id**](HorsesApi.md#delete_horse_by_id) | **DELETE** /horses/{id} | Delete a horse by ID
[**get_horse_by_id**](HorsesApi.md#get_horse_by_id) | **GET** /horses/{id} | Get a horse by ID
[**get_horses**](HorsesApi.md#get_horses) | **GET** /horses | Get all horses
[**update_horse_by_id**](HorsesApi.md#update_horse_by_id) | **PUT** /horses/{id} | Update a horse by ID



## create_horse

> models::Horse create_horse(horse)
Create a new horse

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**horse** | [**Horse**](Horse.md) |  | [required] |

### Return type

[**models::Horse**](Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_horse_by_id

> delete_horse_by_id(id)
Delete a horse by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_horse_by_id

> models::Horse get_horse_by_id(id)
Get a horse by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

[**models::Horse**](Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_horses

> Vec<models::Horse> get_horses()
Get all horses

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Horse>**](Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_horse_by_id

> models::Horse update_horse_by_id(id, horse)
Update a horse by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |
**horse** | [**Horse**](Horse.md) |  | [required] |

### Return type

[**models::Horse**](Horse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

